from datetime import timedelta
import unittest

from pandas import DataFrame

from predictive.dataset import get_heat_session_bounds, get_max_temp_idx


class TestDataset(unittest.TestCase):
    def test_segment_heat_sessions(self):
        self.assertSequenceEqual(
            list(
                get_heat_session_bounds(
                    DataFrame(
                        {
                            "heat_level": [0, 0, 1, 1, 1, 0, 0, 0, 0, 0],
                        }
                    )
                )
            ),
            [(2, 5)],
        )

        self.assertSequenceEqual(
            list(
                get_heat_session_bounds(
                    DataFrame(
                        {
                            "heat_level": [1, 0, 0, 1, 1, 0, 0, 1, 0, 0],
                        }
                    )
                )
            ),
            [(0, 1), (3, 5), (7, 8)],
        )

        self.assertSequenceEqual(
            list(
                get_heat_session_bounds(
                    DataFrame(
                        {
                            "heat_level": [1, 1, 1, 1, 1, 0, 0, 0, 0, 0],
                        }
                    )
                )
            ),
            [(0, 5)],
        )

        self.assertSequenceEqual(
            list(
                get_heat_session_bounds(
                    DataFrame(
                        {
                            "heat_level": [1, 1, 1, 1, 1, 0, 0, 0, 0, 1],
                        }
                    )
                )
            ),
            [(0, 5), (9, 9)],
        )

        self.assertSequenceEqual(
            list(
                get_heat_session_bounds(
                    DataFrame(
                        {
                            "heat_level": [1, 0, 1, 1, 1, 0, 0, 1, 0, 0],
                        }
                    ),
                    threshold=1,
                )
            ),
            [(0, 5), (7, 8)],
        )

        self.assertSequenceEqual(
            list(
                get_heat_session_bounds(
                    DataFrame(
                        {
                            "heat_level": [1, 0, 1, 1, 1, 0, 0, 1, 0, 0],
                        }
                    ),
                    threshold=2,
                )
            ),
            [(0, 8)],
        )

        self.assertSequenceEqual(
            list(
                get_heat_session_bounds(
                    DataFrame(
                        {
                            "heat_level": [1, 0, 1, 1, 0, 0, 0, 1, 0, 0],
                        }
                    ),
                    threshold=3,
                )
            ),
            [(0, 8)],
        )

    def test_get_max_temp_idx(self):
        self.assertEqual(
            get_max_temp_idx(
                DataFrame(
                    {
                        "boiler_temp_c": [5, 5, 5, 5, 5, 5, 6, 6, 6, 7, 7, 7, 7, 7, 7, 7, 7, 7],
                    }
                ),
                start_idx=0,
                threshold=5
            ),
            9,
        )

        self.assertEqual(
            get_max_temp_idx(
                DataFrame(
                    {
                        "boiler_temp_c": [5, 5, 5, 5, 5, 5, 6, 6, 6, 7, 7, 7, 7, 7, 7, 7, 7, 7],
                    }
                ),
                start_idx=0,
                threshold=2
            ),
            0,
        )

        self.assertEqual(
            get_max_temp_idx(
                DataFrame(
                    {
                        "boiler_temp_c": [5, 5, 5, 5, 6, 5, 6, 5, 5, 5, 5],
                    }
                ),
                start_idx=2,
                threshold=2
            ),
            4,
        )
