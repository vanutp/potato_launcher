import os
from pathlib import Path


REPO_ROOT = Path(__file__).absolute().parent.parent
ENV_FILE = REPO_ROOT / 'build.env'
if ENV_FILE.exists():
    _dotenv_contents = dict(
        x.split('=', 1) for x in ENV_FILE.read_text().splitlines() if x
    )
else:
    _dotenv_contents = {}

_sentinel = object()


def get_env(name, default=_sentinel):
    env_val = os.getenv(name)
    # not used intentionally so that empty values are considered missing
    if not env_val:
        env_val = _dotenv_contents.get(name)
    if not env_val:
        if default is _sentinel:
            raise KeyError(f'Config variable {name} is not set')
        return default
    return env_val
