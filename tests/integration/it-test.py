import sys
import os
import subprocess
import time

workdir = "tests/integration"

os.system(f"docker-compose -f {workdir}/docker-compose.it-test.yml up -d ")
while (True):
    proc = subprocess.run(["docker", "exec", "-t", "actix-demo-test-postgres", "pg_isready"],
                          stdout=subprocess.PIPE)
    if proc.stdout.decode('utf-8').endswith("accepting connections\r\n"):
        break
    print("waiting for pg")
    time.sleep(1)

try:
    os.system("cargo test --test integration")
finally:
    os.system(f"docker-compose -f {workdir}/docker-compose.it-test.yml down")
