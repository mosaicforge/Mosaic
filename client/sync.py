import functools
import logging
import warnings

import httpx
import pandas as pd
from pipe import map, traverse

def _client(self):
        """Cached client"""
