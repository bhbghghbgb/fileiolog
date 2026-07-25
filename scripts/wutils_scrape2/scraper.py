"""Scrapy-based spider to download WMI docs from wutils.com.

CLI usage:
    python scraper.py --start-url <url> --depth <N> --output-dir <dir>

Imported usage:
    from scraper import run_spider
    run_spider(start_url="https://wutils.com/wmi/root/wmi/msnt_systemtrace/", max_depth=2)
"""

import argparse
import json
import logging
import os
import re
from urllib.parse import urljoin, urlparse

import scrapy
from scrapy.crawler import CrawlerProcess
from scrapy.spidermiddlewares.httperror import HttpError
from twisted.internet.error import DNSLookupError, ConnectionRefusedError, TimeoutError

logger = logging.getLogger(__name__)

WMI_PATTERN = re.compile(r'/wmi/root/wmi/')


class WmiSpider(scrapy.Spider):
    name = 'wmi_spider'

    def __init__(self, start_url, max_depth=2, output_dir='output', *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.start_urls = [start_url.rstrip('/')]
        self.max_depth = max_depth
        self.output_dir = output_dir
        self.raw_dir = os.path.join(output_dir, 'raw')
        os.makedirs(self.raw_dir, exist_ok=True)

        self.start_breadcrumb_len = None
        self.pages = {}
        self.seen_urls = set()

    def parse(self, response):
        if response.status != 200:
            logger.warning('Non-200 status %s for %s', response.status, response.url)
            return

        if response.url in self.seen_urls:
            return
        self.seen_urls.add(response.url)

        soup = self._get_soup(response)

        breadcrumbs = self._extract_breadcrumbs(soup)
        if not breadcrumbs:
            logger.warning('No breadcrumbs found for %s', response.url)
            return

        if self.start_breadcrumb_len is None:
            self.start_breadcrumb_len = len(breadcrumbs)
            logger.info('Start breadcrumb length: %s (%s)', self.start_breadcrumb_len, ' > '.join(breadcrumbs))

        current_depth = len(breadcrumbs) - self.start_breadcrumb_len
        logger.info('Processing: %s  depth=%s  breadcrumbs=%s', response.url, current_depth, ' > '.join(breadcrumbs))

        filename = self._url_to_filename(response.url)
        filepath = os.path.join(self.raw_dir, filename)
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(f'<!-- Source: {response.url} -->\n')
            f.write(response.text)

        title_tag = soup.find('h1')
        title = title_tag.get_text(strip=True) if title_tag else ''

        self.pages[response.url] = {
            'breadcrumbs': breadcrumbs,
            'file': filename,
            'title': title,
        }

        if current_depth < self.max_depth:
            child_links = self._extract_child_links(soup, response.url)
            logger.info('Found %d child links on %s', len(child_links), response.url)
            for link_url in child_links:
                if link_url not in self.seen_urls:
                    yield scrapy.Request(
                        link_url,
                        callback=self.parse,
                        errback=self._handle_error,
                    )

    def _handle_error(self, failure):
        logger.error('Request failed: %s', failure.request.url)

    def _get_soup(self, response):
        from bs4 import BeautifulSoup
        return BeautifulSoup(response.text, 'lxml')

    def _extract_breadcrumbs(self, soup):
        bc_ul = soup.select_one('ul.breadcrumbs')
        if not bc_ul:
            return []
        return [li.get_text(strip=True) for li in bc_ul.find_all('li')]

    def _extract_child_links(self, soup, base_url):
        html_str = str(soup)
        start_marker = '<!--childs-->'
        end_marker = '<!--/childs-->'
        start = html_str.find(start_marker)
        end = html_str.find(end_marker)
        if start == -1 or end == -1:
            return []

        chunk = html_str[start + len(start_marker):end]
        from bs4 import BeautifulSoup as BS
        local = BS(chunk, 'lxml')

        links = []
        for a in local.find_all('a', href=True):
            href = a['href']
            full = urljoin(base_url, href)
            if WMI_PATTERN.search(full) and full not in links:
                links.append(full)
        return links

    def _url_to_filename(self, url):
        path = urlparse(url).path.rstrip('/')
        last = path.split('/')[-1]
        return f"{last}.html" if last else 'index.html'

    def closed(self, reason):
        mapping = {
            'start_url': self.start_urls[0],
            'start_breadcrumbs': self.pages.get(self.start_urls[0], {}).get('breadcrumbs', []),
            'pages': self.pages,
        }
        mapping_path = os.path.join(self.output_dir, 'mapping.json')
        with open(mapping_path, 'w', encoding='utf-8') as f:
            json.dump(mapping, f, indent=2, ensure_ascii=False)
        logger.info('Saved mapping to %s', mapping_path)
        logger.info('Downloaded %d pages', len(self.pages))


def run_spider(start_url, max_depth=2, output_dir='output'):
    settings = {
        'ROBOTSTXT_OBEY': False,
        'DOWNLOAD_DELAY': 1.0,
        'RANDOMIZE_DOWNLOAD_DELAY': True,
        'CONCURRENT_REQUESTS': 8,
        'RETRY_ENABLED': True,
        'RETRY_TIMES': 3,
        'RETRY_HTTP_CODES': [500, 502, 503, 504, 408, 429],
        'DOWNLOAD_TIMEOUT': 30,
        'LOG_LEVEL': 'INFO',
        'COOKIES_ENABLED': False,
    }
    process = CrawlerProcess(settings)
    process.crawl(WmiSpider, start_url=start_url, max_depth=max_depth, output_dir=output_dir)
    process.start()


def main():
    parser = argparse.ArgumentParser(description='Scrape WMI docs from wutils.com')
    parser.add_argument('--start-url', required=True, help='Starting URL (e.g. https://wutils.com/wmi/root/wmi/msnt_systemtrace/)')
    parser.add_argument('--depth', type=int, default=2, help='Number of levels to follow down from start')
    parser.add_argument('--output-dir', default='output', help='Output directory for raw HTML and mapping')
    args = parser.parse_args()

    logging.basicConfig(level=logging.INFO, format='%(levelname)s %(message)s')
    run_spider(start_url=args.start_url, max_depth=args.depth, output_dir=args.output_dir)


if __name__ == '__main__':
    main()
