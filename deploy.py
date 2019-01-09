#!/usr/bin/env python3

# This script is responsible for deploying this to a VM in the cloud over SSH.

import sys
if sys.version_info[0] < 3:
  raise Exception("Script requires Python 3")

import os, subprocess, configparser

def eprint(*args, **kwargs):
    print(*args, file=sys.stderr, **kwargs)

if __name__ == '__main__':
  # cd into wherever this script is held
  os.chdir(os.path.abspath(os.path.dirname(__file__)))
  
  # Build local code
  returncode = subprocess.call(["cargo", "build", "--release", "--target=x86_64-unknown-linux-musl"], stdout=sys.stdout, stderr=sys.stderr)
  if returncode != 0:
    eprint(f"Error: code did not compile ")
    sys.exit(1)
  
  # Check assumptions
  schedmap_bin = "target/x86_64-unknown-linux-musl/release/schedmap"
  for req_file in [".deployment-id-rsa", ".deployment.ini", "schedmap-deployed.service", schedmap_bin]:
    if not os.path.isfile(req_file):
      eprint(f"Error: file is missing: '{req_file}'")
      sys.exit(1)
  
  config = configparser.ConfigParser()
  config.read(".deployment.ini")
  
  server_host = config["default"]["server"]
  server_user = config["default"]["serveruser"]
  
  user_at_host = f"{server_user}@{server_host}"
  print(f"Deploying {schedmap_bin} to {user_at_host}...")
  
  subprocess.call(["ssh", "-i", ".deployment-id-rsa",
    user_at_host, "sudo", "systemctl", "stop", "schedmap-deployed.service"], stdout=sys.stdout, stderr=sys.stderr)
  subprocess.call(["ssh", "-i", ".deployment-id-rsa",
    user_at_host, "sudo", "chown", "-R", server_user, "/opt/"], stdout=sys.stdout, stderr=sys.stderr)
  
  subprocess.call(["scp", "-i", ".deployment-id-rsa",
    schedmap_bin, f"{user_at_host}:/opt/"], stdout=sys.stdout, stderr=sys.stderr)
  subprocess.call(["scp", "-i", ".deployment-id-rsa",
    "schedmap-deployed.service", f"{user_at_host}:/opt/"], stdout=sys.stdout, stderr=sys.stderr)
  
  subprocess.call(["ssh", "-i", ".deployment-id-rsa",
    user_at_host, "sudo", "cp", "/opt/schedmap-deployed.service", "/usr/lib/systemd/system/schedmap-deployed.service"], stdout=sys.stdout, stderr=sys.stderr)
  subprocess.call(["ssh", "-i", ".deployment-id-rsa",
    user_at_host, "sudo", "systemctl", "enable", "schedmap-deployed.service"], stdout=sys.stdout, stderr=sys.stderr)
  subprocess.call(["ssh", "-i", ".deployment-id-rsa",
    user_at_host, "sudo", "systemctl", "start", "schedmap-deployed.service"], stdout=sys.stdout, stderr=sys.stderr)
  
  
  