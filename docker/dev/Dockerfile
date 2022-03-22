# syntax = edrevo/dockerfile-plus

FROM ubuntu:focal

INCLUDE+ ci/Dockerfile

ENV \
    USERNAME=docker \
    USER_ID=1000 \
    GROUP_ID=1000

# Install a) sudo
#         b) a beginner-friendly text editor
#         c) revision control software
#         d) openssh-client, which includes ssh-keygen for Git
#            users who use SSH-based authentication
#         e) an alternative shell (zsh) that will be later
#            made more functional with ohmyzsh
#         f) a debugger
RUN apt-get install $APT_ARGS \
    sudo nano openssh-client zsh gdb git;

# Discourage root access, this is an interactive instance
#
# We do that by creating a new user that can _on request_
# gain superuser privileges
#
RUN groupadd -g ${GROUP_ID} ${USERNAME}
RUN useradd -u ${USER_ID} -g ${USERNAME} -s /bin/zsh -m -d /home/${USERNAME} ${USERNAME}
RUN usermod -aG sudo ${USERNAME}
RUN echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers

# Disable noninteractive mode
ENV DEBIAN_FRONTEND="dialog"

# Expose libp2p, JSON RPC and websocket ports
EXPOSE 9933 9944 30333

# Switch to non-privileged
USER ${USERNAME}

# Setup OhMyZSH
ENV TERM=xterm \
    HOME=/home/${USERNAME} \
    DISABLE_AUTO_UPDATE=true
RUN wget https://github.com/robbyrussell/oh-my-zsh/raw/master/templates/zshrc.zsh-template -O ${HOME}/.zshrc
RUN wget -O- https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh | sh
WORKDIR /src

# Open shell
CMD [ "zsh" ]
