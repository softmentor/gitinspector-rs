import os
import sys
import yaml

CONFIG_FILENAME = "ktl-myst-plugin.yml"

def log(msg):
    print(f"[ktl-myst-plugin] {msg}", file=sys.stderr)

def load_config(base_path=None):
    """
    Load the YAML config file.
    Searches in base_path (or CWD) and traverses up parent directories
    until the config file is found or the root is reached.
    """
    start_dir = base_path if base_path else os.getcwd()
    current_dir = os.path.abspath(start_dir)
    
    while True:
        config_path = os.path.join(current_dir, CONFIG_FILENAME)
        if os.path.exists(config_path):
            try:
                with open(config_path, 'r') as f:
                    full_config = yaml.safe_load(f) or {}
                    # Store where we found the config to resolve relative paths inside it
                    full_config["_config_dir"] = current_dir
                    log(f"Loaded config from {config_path}")
                    return full_config
            except Exception as e:
                log(f"ERROR: Failed to parse config file at {config_path}: {e}")
                return {}
        
        # Move up one directory
        parent_dir = os.path.dirname(current_dir)
        if parent_dir == current_dir:
            # Reached root
            break
        current_dir = parent_dir
        
    # log(f"WARNING: Config file '{CONFIG_FILENAME}' not found in {start_dir} or its parents.")
    return {}
