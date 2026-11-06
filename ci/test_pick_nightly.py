import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(__file__))
from pick_nightly import parse_title, pick


class PickNightlyTest(unittest.TestCase):
    def test_parses_title(self):
        self.assertEqual(parse_title("Nightly 2026-11-04 \u00b7 6475f62"), ("2026-11-04", "6475f62"))

    def test_rejects_other_titles(self):
        self.assertIsNone(parse_title("Horizon launcher 0.2.41"))

    def test_picks_newest_reachable(self):
        releases = [("nightly", "Nightly 2026-11-04 \u00b7 aaaaaaa"), ("nightly-2026-11-03", "Nightly 2026-11-03 \u00b7 bbbbbbb")]
        self.assertEqual(pick(releases, lambda short: short == "bbbbbbb")["short"], "bbbbbbb")

    def test_none_when_nothing_reachable(self):
        self.assertIsNone(pick([("nightly", "Nightly 2026-11-04 \u00b7 aaaaaaa")], lambda s: False))

    def test_output_shape(self):
        r = pick([("nightly", "Nightly 2026-11-04 \u00b7 6475f62")], lambda s: True)
        self.assertEqual(r, {"tag": "nightly-source/2026-11-04-6475f62", "release": "nightly",
                             "date": "2026-11-04", "short": "6475f62"})


if __name__ == "__main__":
    unittest.main()
