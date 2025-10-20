#!/bin/bash

set -eo pipefail

main() {
  echo "Starting post-create setup in $(pwd)"

  sudo cp ".devcontainer/welcome.txt" "/usr/local/etc/vscode-dev-containers/first-run-notice.txt"
  pre-commit install --install-hooks || { echo "Pre-commit hooks failed to install"; }

  echo "Post-create setup complete!"
}

main "$@"
