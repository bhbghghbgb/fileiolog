"""Orchestrator for wutils WMI doc scraping & conversion.

Runs the Scrapy spider to download HTML, then converts to markdown.

CLI usage:
    python main.py --start-url <url> --depth <N>
    python main.py --scrape-only --start-url <url>
    python main.py --convert-only --raw-dir <dir> --mapping <path>
"""

import argparse
import logging
import sys
import time
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description='WMI docs scraper & converter')
    parser.add_argument('--start-url', default='https://wutils.com/wmi/root/wmi/msnt_systemtrace/',
                        help='Starting URL for scraping')
    parser.add_argument('--depth', type=int, default=2,
                        help='Number of levels to follow down from start')
    parser.add_argument('--output-dir', default='output',
                        help='Output directory for raw HTML and mapping')
    parser.add_argument('--docs-dir', default='docs',
                        help='Output directory for markdown files')
    parser.add_argument('--scrape-only', action='store_true',
                        help='Only run the scraper')
    parser.add_argument('--convert-only', action='store_true',
                        help='Only run the converter')
    parser.add_argument('--mapping', default=None,
                        help='Mapping file path (for --convert-only)')
    parser.add_argument('--raw-dir', default=None,
                        help='Raw HTML directory (for --convert-only)')
    parser.add_argument('--skip-scrape', action='store_true',
                        help='Skip scraping (alias for --convert-only)')
    args = parser.parse_args()

    logging.basicConfig(level=logging.INFO, format='%(levelname)s %(message)s')

    do_scrape = not args.convert_only and not args.skip_scrape
    do_convert = not args.scrape_only

    if do_scrape:
        from scraper import run_spider
        logging.info('=== Starting scraper ===')
        logging.info('URL: %s  Depth: %d  Output: %s', args.start_url, args.depth, args.output_dir)
        run_spider(start_url=args.start_url, max_depth=args.depth, output_dir=args.output_dir)
        logging.info('=== Scraping complete ===')
    else:
        logging.info('Skipping scrape step')

    if do_convert:
        from converter import convert_all
        raw_dir = args.raw_dir or Path(args.output_dir) / 'raw'
        mapping = args.mapping or Path(args.output_dir) / 'mapping.json'
        docs_dir = args.docs_dir

        if not Path(mapping).exists():
            logging.error('Mapping file not found: %s. Run scrape first.', mapping)
            sys.exit(1)

        logging.info('=== Starting conversion ===')
        logging.info('Raw: %s  Mapping: %s  Docs: %s', raw_dir, mapping, docs_dir)
        convert_all(raw_dir=str(raw_dir), mapping_path=str(mapping), output_dir=docs_dir)
        logging.info('=== Conversion complete ===')
    else:
        logging.info('Skipping conversion step')


if __name__ == '__main__':
    main()
