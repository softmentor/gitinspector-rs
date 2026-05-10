import subprocess
import json
import os
import sys

def log(msg):
    print(f"[ktl-myst-plugin] {msg}", file=sys.stderr)

def get_env_path(env_name):
    """
    Finds the path of a conda environment by name.
    Returns path or None.
    """
    try:
        # Running conda env list --json is safer/faster than spawning a run environment
        # Capture stderr too to avoid polluting stdout
        result = subprocess.run(
            ["conda", "env", "list", "--json"], 
            capture_output=True, text=True
        )
        if result.returncode == 0:
            data = json.loads(result.stdout)
            envs = data.get("envs", [])
            for path in envs:
                if os.path.basename(path) == env_name:
                    return path
    except Exception:
        pass
    return None

def check_environment(config, base_path=None):
    """
    Checks if the current python environment matches the configured one.
    """
    python_config = config.get("python", {})
    expected_env = python_config.get("environment", "ktl-env")
    manage_mode = python_config.get("manage", "manual")
    
    # Check Conda environment
    current_env = os.environ.get("CONDA_DEFAULT_ENV")
    
    if current_env == expected_env:
        return

    # Mismatch or no env
    if manage_mode == "auto":
        log(f"Environment mismatch ('{current_env}' != '{expected_env}'). Auto-managing...")
        
        # 1. Check if env exists
        env_path = get_env_path(expected_env)
        
        if not env_path:
            # Look for environment.yml
            env_filename = "environment.yml"
            env_file = env_filename
            
            if base_path:
                 candidate = os.path.join(base_path, env_filename)
                 if os.path.exists(candidate):
                     env_file = candidate
            
            if os.path.exists(env_file):
                log(f"Creating environment '{expected_env}' from {env_file}...")
                try:
                    # Redirect stdout/stderr to sys.stderr to avoid breaking MyST JSON output
                    subprocess.run(["conda", "env", "create", "-f", env_file, "-n", expected_env], check=True, stdout=sys.stderr, stderr=sys.stderr)
                    # Re-resolve path after creation
                    env_path = get_env_path(expected_env)
                except subprocess.CalledProcessError as e:
                    log(f"ERROR: Failed to create environment: {e}")
                    sys.exit(1)
            else:
                search_loc = base_path if base_path else "current directory"
                log(f"WARNING: Environment '{expected_env}' missing and 'environment.yml' not found in {search_loc}. Cannot auto-create.")
        
        if not env_path:
             # Just try falling back to conda run if we couldn't resolve path but maybe it exists?
             # Or just fail/warn
             log(f"WARNING: Could not resolve path for '{expected_env}'. Attempting standard execution...")
             # If we return, we continue in WRONG env.
             return

        # 2. Re-spawn in the correct environment
        script = sys.argv[0]
        args = sys.argv[1:]
        
        log(f"Re-spawning in environment '{expected_env}' at {env_path}...")
        
        # Construct python executable path
        # Windows: env_path/python.exe, Unix: env_path/bin/python
        python_exe = os.path.join(env_path, "bin", "python")
        if not os.path.exists(python_exe):
            python_exe = os.path.join(env_path, "python.exe") # Windows fallback?
        
        if not os.path.exists(python_exe):
             log(f"ERROR: Python executable not found at {python_exe}")
             # Last ditch: conda run?
             sys.exit(1)

        cmd = [python_exe, script] + args
        
        # Ensure the child process knows it's in the correct environment
        child_env = os.environ.copy()
        child_env["CONDA_DEFAULT_ENV"] = expected_env
        
        try:
            # Note: We must flush stdout/stderr before spawning
            sys.stdout.flush()
            sys.stderr.flush()
            # Redirect stderr to stderr (pass through), stdout to stdout
            ret = subprocess.run(cmd, stdin=sys.stdin, stdout=sys.stdout, stderr=sys.stderr, env=child_env)
            sys.exit(ret.returncode)
        except Exception as e:
            log(f"ERROR: Failed to re-spawn in conda environment: {e}")
            sys.exit(1)
