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
  
  # Check assumptions
  for req_file in [".deployment-id-rsa", ".deployment.ini"]:
    if not os.path.isfile(req_file):
      eprint(f"Error: file is missing: '{req_file}'")
      sys.exit(1)
  
  returncode = subprocess.call(["cargo", "build", "--release", "--target=x86_64-unknown-linux-musl"], stdout=sys.stdout, stderr=sys.stderr)
  if returncode != 0:
    eprint(f"Error: code did not compile ")
    sys.exit(1)
  
  config = configparser.ConfigParser()
  config.read(".deployment.ini")
  
  server_host = config["default"]["server"]
  server_user = config["default"]["serveruser"]
  
  print(f"Deploying to {server_user}@{server_host}...")
  
  
  #if not os.path.isfile(""):
    
  