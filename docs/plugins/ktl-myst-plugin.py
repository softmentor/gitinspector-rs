#!/usr/bin/env python3
import sys
import os

# Ensure the local plugin package is in the path
plugin_dir = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, plugin_dir)

def main():
    try:
        from katlas_myst_plugin.cli import main as cli_main
        cli_main()
    except Exception as e:
        import traceback
        print(f"[ktl-myst-plugin] ERROR: {str(e)}", file=sys.stderr)
        traceback.print_exc(file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
