import os
import sys
import pkgutil

def get_css_content():
    """Reads the bundled CSS file from the package static directory."""
    try:
        # Try loading via package resource (best practice if installed)
        # But since we are running via script import in development, 
        # let's try relative path from this file first.
        current_dir = os.path.dirname(os.path.abspath(__file__))
        # ../mermaid/assets/ktl-mermaid.css
        css_path = os.path.join(current_dir, "..", "mermaid", "assets", "ktl-mermaid.css")
        
        if os.path.exists(css_path):
            with open(css_path, "r", encoding="utf-8") as f:
                return f.read()
        
        return None
    except Exception as e:
        print(f"[ktl-myst-plugin] Error reading CSS: {e}", file=sys.stderr)
        return None

def run_css_transform(data, config):
    """
    Injects the CSS into the AST.
    Expected data is the MyST AST root.
    """
    css_content = get_css_content()
    if not css_content:
         print("[ktl-myst-plugin] WARN: ktl-style.css not found.", file=sys.stderr)
         return data

    style_node = {
        "type": "html",
        "value": f'<div style="display: none;"><style>\n/* Katlas MyST Plugin CSS */\n{css_content}\n</style></div>'
    }

    # Inject at the beginning of children, or end? 
    # Usually appending to children is safe.
    # if "children" in data:
    #     data["children"].append(style_node)
    
    # print("[ktl-myst-plugin] Injected ktl-style.css", file=sys.stderr)
    return data
