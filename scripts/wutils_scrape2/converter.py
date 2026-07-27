"""Convert downloaded WMI HTML pages to organised markdown files.

CLI usage:
    python converter.py --raw-dir ./output/raw --mapping ./output/mapping.json --output-dir ./docs

Imported usage:
    from converter import convert_all
    convert_all(raw_dir='output/raw', mapping_path='output/mapping.json', output_dir='docs')
"""

import argparse
import json
import logging
import os
import re
from pathlib import Path
from urllib.parse import urljoin, urlparse

from bs4 import BeautifulSoup, Comment, Tag

logger = logging.getLogger(__name__)

WMI_ROOT = re.compile(r'https?://wutils\.com/wmi/root/wmi/')
WMI_CLASS = re.compile(r'https?://wutils\.com/wmi/root/wmi/[a-z_]+/?')


def load_mapping(mapping_path):
    with open(mapping_path, 'r', encoding='utf-8') as f:
        return json.load(f)


def build_url_map(mapping):
    pages = mapping['pages']
    start_bc = mapping.get('start_breadcrumbs', [])
    start_len = len(start_bc)

    url_map = {}
    for url, info in pages.items():
        bc = info['breadcrumbs']
        root_name = bc[start_len - 1].lower() if start_len > 0 else 'root'

        if len(bc) == start_len:
            output_path = f"{root_name}.md"
        else:
            parts = [p.lower() for p in bc[start_len - 1:]]
            output_path = '/'.join(parts) + '.md'

        clean_url = url.rstrip('/')
        output_path = output_path.replace('\\', '/')
        url_map[clean_url] = output_path
        # Also add root-relative path for browser-saved files
        parsed = urlparse(clean_url)
        if parsed.path:
            url_map[parsed.path] = output_path

    return url_map


def convert_all(raw_dir, mapping_path, output_dir):
    mapping = load_mapping(mapping_path)
    url_map = build_url_map(mapping)

    Path(output_dir).mkdir(parents=True, exist_ok=True)

    total = 0
    for url, info in mapping['pages'].items():
        html_path = os.path.join(raw_dir, info['file'])
        if not os.path.isfile(html_path):
            logger.warning('Missing file: %s', html_path)
            continue

        output_path = os.path.join(output_dir, url_map.get(url.rstrip('/'), info['file'].replace('.html', '.md')))
        os.makedirs(os.path.dirname(output_path), exist_ok=True)

        try:
            markdown = convert_one(html_path, url, url_map)
            with open(output_path, 'w', encoding='utf-8') as f:
                f.write(markdown)
            total += 1
            logger.info('Converted: %s -> %s', info['file'], output_path)
        except Exception as e:
            logger.error('Failed to convert %s: %s', info['file'], e)

    logger.info('Converted %d pages to %s', total, output_dir)


def convert_one(html_path, page_url, url_map):
    with open(html_path, 'r', encoding='utf-8') as f:
        html = f.read()

    soup = BeautifulSoup(html, 'lxml')
    main = soup.select_one('#main-content')
    if not main:
        return '# Error: no #main-content found\n'

    lines = []
    page_url = page_url.rstrip('/')
    page_path = url_map.get(page_url, '')

    lines.append(f'<!-- Source: {page_url} -->')
    lines.append('')

    title = _extract_title(main)
    if title:
        lines.append(f'# {title}')
        lines.append('')

    bc_text = _extract_breadcrumbs_text(soup)
    if bc_text:
        lines.append(f'**Breadcrumbs:** {bc_text}')
        lines.append('')

    description = _extract_description(main)
    if description:
        lines.append(description)
        lines.append('')

    childs = _extract_child_classes(soup, url_map, page_path)
    if childs:
        lines.append(childs)
        lines.append('')

    properties = _extract_properties(soup, main, url_map, page_path)
    if properties:
        lines.append(properties)
        lines.append('')

    qualifiers = _extract_qualifiers(soup, main)
    if qualifiers:
        lines.append(qualifiers)
        lines.append('')

    system_props = _extract_system_properties(soup, main)
    if system_props:
        lines.append(system_props)
        lines.append('')

    return '\n'.join(lines)


def _content_between(soup, start_marker, end_marker):
    html = str(soup)
    s = html.find(start_marker)
    e = html.find(end_marker)
    if s == -1 or e == -1 or e <= s:
        return None
    return html[s + len(start_marker):e]


def _table_to_markdown(table_tag, url_map=None, skip_rows=0, current_file=''):
    rows = table_tag.find_all('tr')
    if len(rows) <= skip_rows:
        return None

    data = []
    for tr in rows[skip_rows:]:
        cells = []
        for cell in tr.find_all(['th', 'td']):
            links = cell.find_all('a')
            if links:
                parts = []
                for a in links:
                    text = a.get_text(strip=True)
                    href = a.get('href', '')
                    if text:
                        parts.append(f'[{text}]({_rewrite_url(href, url_map, current_file)})')
                # include any non-link text
                for t in cell.find_all(string=True, recursive=False):
                    t = t.strip()
                    if t:
                        parts.append(t)
                cells.append(' '.join(parts))
            else:
                cells.append(cell.get_text(strip=True))
        data.append(cells)

    if not data:
        return None

    col_count = max(len(r) for r in data)
    out = []
    header = data[0]
    out.append('| ' + ' | '.join(h.replace('\n', ' ') for h in header[:col_count]) + ' |')
    out.append('| ' + ' | '.join('---' for _ in range(col_count)) + ' |')
    for row in data[1:]:
        while len(row) < col_count:
            row.append('')
        out.append('| ' + ' | '.join(r.replace('\n', ' ') for r in row[:col_count]) + ' |')
    return '\n'.join(out)


def _rewrite_url(href, url_map, current_file=''):
    if not href:
        return ''
    if href.startswith('#'):
        return href
    if not url_map:
        return href

    anchor = ''
    if '#' in href:
        idx = href.index('#')
        anchor = href[idx:]
        base = href[:idx]
    else:
        base = href

    stripped = base.rstrip('/')
    target = None

    if stripped in url_map:
        target = url_map[stripped]
    else:
        # Match absolute wutils.com URLs
        m = re.match(r'https?://wutils\.com(/wmi/root/wmi/([a-z0-9_]+))/?', stripped)
        # Match root-relative URLs like /wmi/root/wmi/alpc
        if not m:
            m = re.match(r'(/wmi/root/wmi/([a-z0-9_]+))/?' , stripped)
        if m:
            full_path = m.group(1)
            class_name = m.group(2).lower()
            if full_path in url_map:
                target = url_map[full_path]
            else:
                target = class_name + '.md'

    if target:
        if anchor and target == current_file:
            return anchor
        if current_file:
            cur_dir = os.path.dirname(current_file) if '/' in current_file else ''
            rel = os.path.relpath(target, start=cur_dir).replace('\\', '/')
        else:
            rel = target
        if anchor:
            return f'{rel}{anchor}'
        return rel

    for prefix in ['https://wutils.com', 'http://wutils.com']:
        if stripped.startswith(prefix):
            relative = stripped[len(prefix) + 1:]
            return relative + anchor if anchor else relative
    return href


def _extract_title(main):
    h1 = main.find('h1')
    return h1.get_text(strip=True) if h1 else None


def _extract_breadcrumbs_text(soup):
    ul = soup.select_one('ul.breadcrumbs')
    if not ul:
        return None
    items = []
    for a in ul.find_all('a'):
        items.append(a.get_text(strip=True))
    return ' > '.join(items) if items else None


def _extract_description(main):
    desc_h2 = main.find('h2', string=lambda t: t and t.strip() == 'Description')
    if not desc_h2:
        return None

    parts = []
    for sib in desc_h2.find_next_siblings():
        if isinstance(sib, Comment):
            if '****' in str(sib):
                break
            continue
        if sib.name and sib.name.startswith('h'):
            break
        text = sib.get_text(strip=True)
        if text:
            parts.append(text)
    text = ' '.join(parts)
    return text if text else None


def _extract_child_classes(soup, url_map, current_file=''):
    raw = _content_between(soup, '<!--childs-->', '<!--/childs-->')
    if not raw:
        return None

    local = BeautifulSoup(raw, 'lxml')

    h2 = local.find('h2')
    heading = h2.get_text(strip=True) if h2 else 'Child Classes'

    for div in local.find_all(['div', 'p']):
        text = div.get_text(strip=True)
        if 'Number of classes' in text:
            div.decompose()

    table = local.find('table')
    md_table = _table_to_markdown(table, url_map, current_file=current_file) if table else None
    if not md_table:
        return None

    out = [f'## {heading}', '', md_table]
    return '\n'.join(out)


def _extract_properties(soup, main, url_map, current_file=''):
    raw = _content_between(soup, '<!--properties-->', '<!--/properties-->')
    if not raw:
        return None

    parts = []

    local = BeautifulSoup(raw, 'lxml')
    h2 = local.find('h2')
    if h2:
        parts.append(f'## {h2.get_text(strip=True)}')
        parts.append('')

    table = local.find('table')
    if table:
        md = _table_to_markdown(table, url_map, current_file=current_file)
        if md:
            parts.append(md)
            parts.append('')

    props_end = str(soup).find('<!--/properties-->')
    quals_start = str(soup).find('<!--qualifiers-->')
    if props_end != -1 and quals_start != -1:
        detail_html = str(soup)[props_end + len('<!--/properties-->'):quals_start]
        detail_soup = BeautifulSoup(detail_html, 'lxml')

        detail_tables = detail_soup.find_all('table')
        for dt in detail_tables:
            dt_md = _detail_table_to_markdown(dt, url_map, current_file)
            if dt_md:
                parts.append(dt_md)
                parts.append('')

    return '\n'.join(parts) if parts else None


def _detail_table_to_markdown(table_tag, url_map, current_file=''):
    rows = table_tag.find_all('tr')
    if not rows:
        return None

    out_parts = []

    title_row = rows[0] if rows else None
    if title_row:
        th = title_row.find('th', colspan=True)
        if th:
            title_text = th.get_text(strip=True).lstrip('\u25b2').strip()
            out_parts.append(f'### {title_text}')
            out_parts.append('')

    data_rows = []
    for tr in rows[1:]:
        cells = tr.find_all(['th', 'td'])
        if len(cells) == 2:
            key = cells[0].get_text(strip=True)
            val_cell = cells[1]
            value = _cell_to_text(val_cell, url_map, current_file)
            data_rows.append((key, value))

    if data_rows:
        out_parts.append('| Field | Value |')
        out_parts.append('|-------|-------|')
        for k, v in data_rows:
            out_parts.append(f'| {k} | {v} |')

    return '\n'.join(out_parts) if out_parts else None


def _cell_to_text(cell, url_map, current_file=''):
    lines = []
    for child in cell.children:
        if isinstance(child, Tag):
            if child.name == 'br':
                lines.append('<br>')
            elif child.name == 'a':
                text = child.get_text(strip=True)
                href = child.get('href', '')
                if href and text:
                    lines.append(f'[{text}]({_rewrite_url(href, url_map, current_file)})')
                elif text:
                    lines.append(text)
            else:
                t = child.get_text(strip=True)
                if t:
                    lines.append(t)
        elif child.strip():
            lines.append(child.strip())
    text = ' '.join(lines)
    text = re.sub(r"<br>\s*<br>", ' ', text)
    text = text.replace('<br>', ' ')
    text = re.sub(r'\s+', ' ', text).strip()
    return text


def _extract_qualifiers(soup, main):
    raw = _content_between(soup, '<!--qualifiers-->', '<!--/qualifiers-->')
    if not raw:
        return None

    local = BeautifulSoup(raw, 'lxml')
    parts = []

    h2 = local.find('h2')
    if h2:
        parts.append(f'## {h2.get_text(strip=True)}')
        parts.append('')

    table = local.find('table')
    if table:
        md = _table_to_markdown(table)
        if md:
            parts.append(md)

    return '\n'.join(parts) if parts else None


def _extract_system_properties(soup, main):
    quals_end = str(soup).find('<!--/qualifiers-->')
    if quals_end == -1:
        return None

    rest = str(soup)[quals_end + len('<!--/qualifiers-->'):]

    similar_idx = rest.find('<h2>Similar Classes')
    if similar_idx != -1:
        rest = rest[:similar_idx]
    disqus_idx = rest.find('<div id="disqus_thread"')
    if disqus_idx != -1:
        rest = rest[:disqus_idx]

    rest = rest.strip()
    if not rest:
        return None

    local = BeautifulSoup(rest, 'lxml')
    parts = []

    h2 = local.find('h2')
    if h2:
        parts.append(f'## {h2.get_text(strip=True)}')
        parts.append('')

    table = local.find('table')
    if table:
        md = _table_to_markdown(table)
        if md:
            parts.append(md)

    return '\n'.join(parts) if parts else None


def convert_one_cli():
    parser = argparse.ArgumentParser(description='Convert a single HTML file to markdown')
    parser.add_argument('html_file', help='Path to HTML file')
    parser.add_argument('--output', '-o', help='Output markdown file path')
    parser.add_argument('--mapping', help='Path to mapping.json for URL rewriting')
    args = parser.parse_args()

    url_map = {}
    if args.mapping:
        mapping = load_mapping(args.mapping)
        url_map = build_url_map(mapping)

    md = convert_one(args.html_file, 'https://wutils.com/wmi/root/wmi/', url_map)
    if args.output:
        with open(args.output, 'w', encoding='utf-8') as f:
            f.write(md)
        print(f'Written to {args.output}')
    else:
        print(md)


def main():
    parser = argparse.ArgumentParser(description='Convert downloaded WMI HTML files to markdown')
    parser.add_argument('--raw-dir', default='output/raw', help='Directory with raw HTML files')
    parser.add_argument('--mapping', default='output/mapping.json', help='Path to mapping.json')
    parser.add_argument('--output-dir', default='docs', help='Output directory for markdown files')
    args = parser.parse_args()

    logging.basicConfig(level=logging.INFO, format='%(levelname)s %(message)s')
    convert_all(raw_dir=args.raw_dir, mapping_path=args.mapping, output_dir=args.output_dir)


if __name__ == '__main__':
    main()
