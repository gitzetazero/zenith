import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(__file__))
from check_launch_api import NEEDED, problems


def api(version="0.1.0", drop=()):
    return {"launch_api": version, "options": [{"name": o, "value": "X"} for o in NEEDED if o not in drop] + [{"name": "--fullscreen", "value": None}]}


class CheckLaunchApiTest(unittest.TestCase):
    def test_complete_api_passes(self):
        self.assertEqual(problems(api()), [])

    def test_newer_minor_of_0_1_passes(self):
        self.assertEqual(problems(api("0.1.7")), [])

    def test_other_line_fails(self):
        self.assertEqual(len(problems(api("0.2.0"))), 1)

    def test_each_missing_option_fails(self):
        for o in NEEDED:
            self.assertEqual(problems(api(drop=(o,))), [f"option {o} is missing"])

    def test_option_without_argument_is_found(self):
        self.assertEqual(problems({"launch_api": "0.1.0", "options": [{"name": o, "value": None} for o in NEEDED]}), [])


if __name__ == "__main__":
    unittest.main()
