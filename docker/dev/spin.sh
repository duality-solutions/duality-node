#!/usr/bin/env bash

# Sanity checks
if ! [[ "$(uname)" == 'Linux' ]]; then
  echo "$0: This script may not be running on a Linux-based operating system, exiting!" >&2; exit 1;
fi

if ! [ -x "$(command -v docker)" ] || ! [ -x "$(command -v docker-compose)" ]; then
  echo "$0: docker and/or docker-compose is not detected. Is it included with \$PATH?" >&2;  exit 1;
fi

if ! [ -f "$(pwd)/docker-compose.yml" ]; then
  echo "$0: Unable to locate docker-compose.yml in current directory. Are you sure you're in the correct folder?" >&2;  exit 1;
fi

# Environment Variables that we declare until Docker makes
# Docker BuildKit opt-out instead of opt-in 
export DOCKER_BUILDKIT=1
export COMPOSE_DOCKER_CLI_BUILD=1

# Internal global state
export INHERIT_GIT_VARIABLES=0
export INHERIT_READONLY_SSHD=0
export INHERIT_CRGO_REGCACHE=0
export SEND_HELP_AND_SAY_BYE=0
export DOCKER_ADDITIONALARGS=""

# This could change if someone modifies the Dockerfile 
# so let's keep this as an option
export PRESUMED_HOME="/home/docker" 

# shellcheck disable=SC2295
# https://stackoverflow.com/a/20460402
function string_contain() { [ -z "$1" ] || { [ -z "${2##*$1*}" ] && [ -n "$2" ];};}

# Parse arguments, if any and then proceed ahead, everything is opt-in
# So negatives aren't considered by the parser as such options do not exist
for arg in "$@"
do
    INPUT_VERB="$(echo "$arg" | tr '[:upper:]' '[:lower:]' | cut -c 1-)"
    if string_contain 'i' "$INPUT_VERB"; then INHERIT_GIT_VARIABLES=1; fi;
    if string_contain 's' "$INPUT_VERB"; then INHERIT_READONLY_SSHD=1; fi;
    if string_contain 'c' "$INPUT_VERB"; then INHERIT_CRGO_REGCACHE=1; fi;
    if string_contain 'h' "$INPUT_VERB"; then SEND_HELP_AND_SAY_BYE=1; fi;
done

if ! [ "$SEND_HELP_AND_SAY_BYE" -eq 0 ]; then
    echo "$0: spin up a Docker container where you can build on Substrate!";
    echo "";
    echo "$0 [arguments]";
    echo "";
    echo "List of valid arguments:";
    echo "       -i   - Inherit host Git repo-specific author name and email address";
    echo "       -s   - Map read-only access from ~/.ssh to container";
    echo "       -c   - Map read-write access from $(whoami)'s crates cache to the container ";
    echo " -h or -?   - Print this help message";
    echo ""
    exit 0;
fi

# Inherit Git environment variables from host if we have Git installed
if ! [ "$INHERIT_GIT_VARIABLES" -eq 0 ] && [ -x "$(command -v git)" ] ; then
    GIT_LOCAL_AUTHORNAME="$(git config user.name)"
    GIT_LOCAL_AUTHORMAIL="$(git config user.email)"
    DOCKER_ADDITIONALARGS="$DOCKER_ADDITIONALARGS -e GIT_AUTHOR_NAME=\"$GIT_LOCAL_AUTHORNAME\" -e GIT_COMMITTER_NAME=\"$GIT_LOCAL_AUTHORNAME\" -e GIT_AUTHOR_EMAIL=\"$GIT_LOCAL_AUTHORMAIL\" -e GIT_COMMITTER_EMAIL=\"$GIT_LOCAL_AUTHORMAIL\" -e EMAIL=\"$GIT_LOCAL_AUTHORMAIL\""
fi

# Opportunistically check if directories exist and 
# mount them if we can
#
# read-only:  $HOME/.ssh
# read-write: $CARGO_HOME/bin/
#                        /git/db/
#                        /registry/index/
#                        /registry/cache/
#
if ! [ "$INHERIT_READONLY_SSHD" -eq 0 ] &&  [ -d "$HOME/.ssh" ]; then
    DOCKER_ADDITIONALARGS="$DOCKER_ADDITIONALARGS -v $HOME/.ssh:$PRESUMED_HOME/.ssh:ro"
fi

if ! [ -d "$CARGO_HOME" ]; then
    CARGO_HOME="$HOME/.cargo"
fi

if ! [ "$INHERIT_CRGO_REGCACHE" -eq 0 ]; then
    if [ -d "$CARGO_HOME/bin" ] && [ -d "$CARGO_HOME/git/db" ] && [ -d "$CARGO_HOME/registry/index" ] && [ -d "$CARGO_HOME/registry/cache" ]; then 
        DOCKER_ADDITIONALARGS="$DOCKER_ADDITIONALARGS -v \"$CARGO_HOME/bin:$PRESUMED_HOME/.cargo/bin\""
        DOCKER_ADDITIONALARGS="$DOCKER_ADDITIONALARGS -v \"$CARGO_HOME/git/db:$PRESUMED_HOME/.cargo/git/db\""
        DOCKER_ADDITIONALARGS="$DOCKER_ADDITIONALARGS -v \"$CARGO_HOME/registry/index:$PRESUMED_HOME/.cargo/registry/index\"";
        DOCKER_ADDITIONALARGS="$DOCKER_ADDITIONALARGS -v \"$CARGO_HOME/registry/cache:$PRESUMED_HOME/.cargo/registry/cache\""; 
    fi
fi

SHELL_EXEC="docker-compose run $DOCKER_ADDITIONALARGS container"

# Start building the container
docker-compose build

# Finally, run the container
echo "Running '$SHELL_EXEC' in bash subshell..."
bash -c "$SHELL_EXEC"
