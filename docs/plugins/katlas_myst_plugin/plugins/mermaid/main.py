#!/usr/bin/env python3
import sys
import json
import yaml
import os
import urllib.request
import argparse
import copy
from jsonschema import validate, ValidationError

SCHEMA_URL = "https://mermaid.js.org/schemas/config.schema.json"
SCHEMA_CACHE_FILE = "mermaid.schema.json"

def log(msg):
    """Log to stderr so it shows up in MyST build logs."""
    print(f"[KatlasMermaidPlugin] {msg}", file=sys.stderr)

def get_schema():
    """Fetch and cache the Mermaid config schema."""
    script_dir = os.path.dirname(os.path.abspath(__file__))
    cache_path = os.path.join(script_dir, SCHEMA_CACHE_FILE)
    
    if os.path.exists(cache_path):
        try:
            with open(cache_path, 'r') as f:
                return json.load(f)
        except:
            log("Invalid cache, re-fetching schema.")
    
    try:
        log(f"Fetching schema from {SCHEMA_URL}...")
        with urllib.request.urlopen(SCHEMA_URL) as response:
            schema = json.loads(response.read().decode())
            with open(cache_path, 'w') as f:
                json.dump(schema, f)
            return schema
    except Exception as e:
        log(f"WARNING: Failed to fetch schema: {e}")
        return None

def deep_merge(source, destination):
    """
    Deep merge two dictionaries.
    'source' is the base config (Global).
    'destination' is the override config (Local).
    """
    for key, value in source.items():
        if isinstance(value, dict):
            node = destination.setdefault(key, {})
            deep_merge(value, node)
        else:
            if key not in destination:
                destination[key] = value
    return destination



def create_mermaid_node(original_node, config, extra_class):
    """
    Creates a new Mermaid node with the given config injected.
    Wraps it in a container div with the specified class.
    """
    # 1. Prepare Content
    current_value = original_node.get("value", "")
    local_config = {}
    diagram_code = current_value
    
    # Parse existing frontmatter
    if current_value.strip().startswith("---"):
        try:
            parts = current_value.split("---", 2)
            if len(parts) >= 3:
                frontmatter_yaml = parts[1]
                diagram_code = parts[2]
                parsed = yaml.safe_load(frontmatter_yaml)
                if parsed and "config" in parsed:
                    local_config = parsed["config"]
        except Exception:
            pass 
    
    # Merge Configs
    merged_config = copy.deepcopy(local_config)
    deep_merge(config, merged_config)
    
    # Remove custom internal keys
    if "katlas" in merged_config:
        del merged_config["katlas"]
    
    # Cleanup for Dark Mode
    current_theme = merged_config.get("theme", "default")
    if current_theme != "base":
        if "themeVariables" in merged_config:
            new_vars = {}
            for k, v in merged_config["themeVariables"].items():
                if isinstance(v, str) and v.startswith("var(--"):
                        pass
                else:
                    new_vars[k] = v
            if not new_vars:
                del merged_config["themeVariables"]
            else:
                merged_config["themeVariables"] = new_vars

    # Restore Validation
    try:
        schema = get_schema()
        if schema:
            validate(instance=merged_config, schema=schema)
    except ValidationError as e:
        log(f"Schema Validation Error: {e.message}")

    # Reconstruct Frontmatter
    try:
        yml_str = yaml.dump(merged_config, default_flow_style=False)
        indented_yml = "\n".join(["  " + line for line in yml_str.splitlines()])
        frontmatter = f"---\nconfig:\n{indented_yml}\n---\n"
        final_value = frontmatter + diagram_code.strip()
    except Exception as e:
        log(f"Error constructing frontmatter: {e}")
        final_value = current_value 

    # 2. Create Node
    mermaid_code_node = copy.deepcopy(original_node)
    mermaid_code_node["type"] = "mermaid"
    mermaid_code_node["value"] = final_value
    
    if extra_class:
        wrapper_node = {
            "type": "container",
            "kind": "div",
            "class": extra_class,
            "children": [mermaid_code_node]
        }
        return wrapper_node
    else:
        return mermaid_code_node

def transform_nodes(node, global_config, katlas_settings):
    """
    Recursively transforms the AST.
    Returns a LIST of nodes to replace the current node.
    """
    # Check if it's a mermaid node or code block
    is_mermaid = False
    node_type = node.get("type", "")
    if node_type == "mermaid":
        is_mermaid = True
    elif node_type == "code" and node.get("lang", "").strip().lower() == "mermaid":
        is_mermaid = True
    
    if is_mermaid:
        # Check dual render setting
        dual_render = katlas_settings.get("dualRender", True)
        
        if dual_render:
            # 1. Light Mode Config
            light_run_config = copy.deepcopy(global_config)
            
            # 2. Dark Mode Config
            dark_run_config = copy.deepcopy(global_config)
            dark_run_config["theme"] = "dark" 
            
            # Output nodes
            light_node = create_mermaid_node(node, light_run_config, "mermaid-light")
            dark_node = create_mermaid_node(node, dark_run_config, "mermaid-dark")
            
            # Extract identifier to surface it to the transclusion wrapper
            ident = node.get("identifier")
            lbl = node.get("label")
            html_id = node.get("html_id")

            # Clean all internal identifiers so we don't duplicate them in AST
            for target_node in [light_node, dark_node]:
                if "identifier" in target_node: del target_node["identifier"]
                if "label" in target_node: del target_node["label"]
                if "html_id" in target_node: del target_node["html_id"]
                if "children" in target_node and len(target_node["children"]) > 0:
                    child = target_node["children"][0]
                    if "identifier" in child: del child["identifier"]
                    if "label" in child: del child["label"]
                    if "html_id" in child: del child["html_id"]
            
            # Create a uniform transclusion wrapper carrying the label
            wrapper_node = {
                "type": "container",
                "kind": "div",
                "class": "katlas-mermaid-dual-container",
                "children": [light_node, dark_node]
            }

            if ident: wrapper_node["identifier"] = ident
            if lbl: wrapper_node["label"] = lbl
            if html_id: wrapper_node["html_id"] = html_id
            
            return [wrapper_node]
        else:
            single_config = copy.deepcopy(global_config)
            processed_node = create_mermaid_node(node, single_config, None)
            return [processed_node]

    # Recurse into children
    if "children" in node:
        new_children = []
        for child in node["children"]:
            replacements = transform_nodes(child, global_config, katlas_settings)
            new_children.extend(replacements)
        node["children"] = new_children
    
    return [node]

def load_global_mermaid_config(config_path=None, config_dir=None):
    """
    Loads Mermaid config from a file path or falls back to the default package asset.
    """
    loaded_config = {}
    config_dir = config_dir or os.getcwd()
    
    # 1. Try User Config
    if config_path:
        # Resolve config_path relative to the config directory 
        abs_path = os.path.abspath(os.path.join(config_dir, config_path))
        if os.path.exists(abs_path):
            try:
                with open(abs_path, 'r') as f:
                    loaded_config = yaml.safe_load(f) or {}
                log(f"Loaded global mermaid config from {abs_path}")
                return loaded_config
            except Exception as e:
                log(f"WARNING: Failed to load mermaid config at {abs_path}: {e}")
        else:
            log(f"WARNING: Mermaid config file not found at {abs_path}")

    # 2. Fallback to Default (Bundled)
    try:
        # Try relative path first (dev mode)
        script_dir = os.path.dirname(os.path.abspath(__file__))
        # Adjusted path for new structure: ../../../static? No, assets are now in assets/
        default_path = os.path.join(script_dir, "assets", "default_mermaid_config.yml")
        
        if os.path.exists(default_path):
             with open(default_path, 'r') as f:
                loaded_config = yaml.safe_load(f) or {}
             log(f"Loaded default mermaid config from {default_path}")
        else:
             log("WARNING: Default mermaid config not found in package.")
    except Exception as e:
        log(f"WARNING: Failed to load default config: {e}")
        
    return loaded_config

def run_mermaid_transform(data, full_config):
    """
    Main entry point for Mermaid transformation.
    """
    full_config = full_config or {}
    
    # 1. Load Mermaid Global Config
    mermaid_plugin_config = full_config.get("diagrams", {}).get("mermaid", {})
    global_config_path = mermaid_plugin_config.get("global_config")
    
    config_dir = full_config.get("_config_dir", os.getcwd())
    mermaid_config = load_global_mermaid_config(global_config_path, config_dir)

    # 1.1 Merge Overrides from ktl-myst-plugin.yml
    # The configuration in ktl-myst-plugin.yml (mermaid_plugin_config) should override defaults.
    overrides = copy.deepcopy(mermaid_plugin_config)
    # Remove control keys that aren't part of Mermaid config
    if "global_config" in overrides:
        del overrides["global_config"]
    if "enabled" in overrides:
        del overrides["enabled"]
    
    # helper deep_merge function is available in this file
    # deep_merge(source=base, destination=override)
    # We want overrides to win, so we merge base (mermaid_config) into overrides
    # Then overrides becomes the new master config.
    mermaid_config = deep_merge(mermaid_config, overrides)

    # 2. Extract Katlas Settings
    # Prioritize settings in mermaid-global-config.yml (under 'katlas') if merged?
    # Or keep them separate.
    
    # Let's check the loaded mermaid config for 'katlas' settings
    katlas_settings = mermaid_config.pop("katlas", {})
    
    # Allow overrides from ktl-myst-plugin.yml (if any are relevant in future)
    # But for now, we respect the loaded config first.
    
    if "dualRender" not in katlas_settings:
         katlas_settings["dualRender"] = True 

    if "children" in data:
       new_children = []
       for child in data["children"]:
           # First, check if this is a ktl:mermaid directive and convert it
           processed_child = child
           if child.get("type") == "mystDirective" and child.get("name") == "ktl:mermaid":
               log(f"Found ktl:mermaid directive: {json.dumps(child, default=str)}")
               # Convert to mermaid node
               processed_child = {
                   "type": "mermaid",
                   "value": child.get("value", "") 
               }
               # Check if child has children that might contain the code if value is empty
               if not processed_child["value"] and "children" in child:
                   # This is a fallback/guess. Usually for code directives value is set.
                   pass
               
               log(f"Converted to: {json.dumps(processed_child, default=str)}")
               
           replacements = transform_nodes(processed_child, mermaid_config, katlas_settings)
           new_children.extend(replacements)
       data["children"] = new_children
       
    return data

PLUGIN_SPEC = {
    "name": "Katlas Mermaid Plugin",
    "directives": [],
    "transforms": [
        {
            "stage": "document",
            "plugin": "katlas-mermaid-transform"
        }
    ]
}

def main():
    parser = argparse.ArgumentParser()
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--transform", action="store_true") 
    parser.add_argument("--format", default="json")
    
    args, unknown = parser.parse_known_args()

    if args.transform:
        try:
            log("Starting transform...")
            input_data = sys.stdin.read()
            log(f"Read {len(input_data)} bytes from stdin")
            if input_data.strip():
                data = json.loads(input_data)
                result = run_mermaid_transform(data)
                output_str = json.dumps(result)
                log(f"Transform complete, writing {len(output_str)} bytes to stdout")
                sys.stdout.write(output_str)
            else:
                log("Empty input data")
                pass
        except Exception as e:
            log(f"Critical Error during transform: {e}")
            sys.exit(1)
    else:
        print(json.dumps(PLUGIN_SPEC))

if __name__ == "__main__":
    main()
