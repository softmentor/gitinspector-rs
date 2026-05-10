#!/usr/bin/env python3
import sys
import os
import subprocess
import shutil

# This is a thin wrapper script for the Katlas MyST Plugin.
# Its purpose is to bootstrap the environment and execute the installed plugin.

def is_venv_active():
    return sys.prefix != sys.base_prefix or hasattr(sys, "real_prefix") or os.environ.get("CONDA_DEFAULT_ENV")

def main():
    # 1. Check Environment
    # We ideally want to be running inside the project's configured environment.
    # But this script is the entry point, MyST calls it.
    
    try:
        from katlas_myst_plugin.cli import main as cli_main
    except ImportError:
        # Plugin not found.
        # Check if we are in a virtual environment
        if not is_venv_active():
            print("[ktl-myst-plugin] ERROR: The 'katlas-myst-plugin' package is not found and no virtual environment seems active.", file=sys.stderr)
            print("[ktl-myst-plugin] Please activate your environment: `conda activate ktl-env` or `source venv/bin/activate`", file=sys.stderr)
            sys.exit(1)
        else:
             print(f"[ktl-myst-plugin] ERROR: The 'katlas-myst-plugin' package is not installed in the current environment ({sys.executable}).", file=sys.stderr)
             print("[ktl-myst-plugin] Please install it: `pip install katlas-myst-plugin`", file=sys.stderr)
             sys.exit(1)

    # 2. Execute Code Plugin
    cli_main()

if __name__ == "__main__":
    main()
