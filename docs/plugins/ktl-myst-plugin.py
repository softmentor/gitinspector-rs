#!/usr/bin/env python3
import sys
import os

# This is a thin wrapper script for the Katlas MyST Plugin.
# Its purpose is to bootstrap the environment and execute the installed plugin.

def main():
    # 1. Local Source Support
    # If we find the source code relative to this script, we prioritize it.
    # This enables "Bundled" mode (useful for development and self-contained projects).
    script_dir = os.path.dirname(os.path.abspath(__file__))
    
    # Check common locations for the bundled package
    local_source_paths = [
        os.path.join(script_dir, "plugins"),
        os.path.join(script_dir, "katlas_myst_plugin"), # If someone copied the package directly
        os.path.join(script_dir, "..", "python", "katlas-myst-plugin", "src"), # Dev mode
    ]
    
    for path in local_source_paths:
        if os.path.exists(path):
            sys.path.insert(0, path)
            break

    # 2. Execute the Plugin
    try:
        from katlas_myst_plugin.cli import main as cli_main
        cli_main()
    except ImportError as e:
        print(f"[ktl-myst-plugin] ERROR: The 'katlas-myst-plugin' package or its dependencies (pyyaml, jsonschema) are not found.", file=sys.stderr)
        print(f"[ktl-myst-plugin] Debug: {str(e)}", file=sys.stderr)
        print("[ktl-myst-plugin] Please ensure dependencies are installed: `pip install pyyaml jsonschema`", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"[ktl-myst-plugin] CRITICAL ERROR: {str(e)}", file=sys.stderr)
        import traceback
        traceback.print_exc(file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
