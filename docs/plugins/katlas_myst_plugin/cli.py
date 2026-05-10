import argparse
import sys
import json
import os

from .core.config import load_config, log
from .core.environment import check_environment
from .plugins.mermaid.main import run_mermaid_transform

# Definition of the plugin capabilities
PLUGIN_SPEC = {
    "name": "Katlas MyST Plugin",
    "directives": [
        {
            "name": "ktl:mermaid",
            "doc": "Mermaid diagram directive",
            "body": {"type": "string"},
            "arg": {"type": "string"}
        }
    ],
    "transforms": [
        {
            "name": "katlas-mermaid",
            "stage": "document"
        }
    ]
}

def main():
    parser = argparse.ArgumentParser()
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--transform") 
    group.add_argument("--directive") 
    parser.add_argument("--format", default="json")
    parser.add_argument("--config-dir", help="Base directory to search for config", default=None)
    
    args, unknown = parser.parse_known_args()

    # 1. Directive Handling (stdin -> stdout)
    if args.directive == 'ktl:mermaid':
        try:
            input_data = sys.stdin.read()
            if input_data.strip():
                data = json.loads(input_data)
                # Return a generic 'mermaid' node
                node = {
                    "type": "mermaid",
                    "value": data.get("body", "")
                }
                print(json.dumps([node]))
            else:
                 print(json.dumps([]))
        except Exception as e:
            log(f"Error in directive: {e}")
            sys.exit(1)

    # 2. Transform Handling (stdin -> stdout)
    elif args.transform:
        # Load global config once
        config = load_config(args.config_dir)
        
        # Verify environment 
        env_base = config.get("_config_dir") or os.getcwd()
        check_environment(config, base_path=env_base)

        try:
            input_data = sys.stdin.read()
            if input_data.strip():
                data = json.loads(input_data)
                
                if args.transform == 'katlas-mermaid':
                    result = run_mermaid_transform(data, config)
                    print(json.dumps(result))
                else:
                    # Unknown transform
                    print(json.dumps(data))
            else:
                pass
        except Exception as e:
            log(f"Critical Error during transform: {e}")
            sys.exit(1)
            
    # 3. Spec Output (default)
    else:
        print(json.dumps(PLUGIN_SPEC))

if __name__ == "__main__":
    main()
