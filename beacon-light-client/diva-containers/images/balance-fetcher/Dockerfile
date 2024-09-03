FROM diva-base

# Compile contracts.
RUN nix develop --accept-flake-config --extra-experimental-features 'nix-command flakes' --command sh -c "cd /app/beacon-light-client/solidity && yarn hardhat compile"

ARG AWS_ACCESS_KEY_ID
ARG AWS_SECRET_ACCESS_KEY

RUN mkdir ~/.aws
RUN echo '[default]'                                      >> ~/.aws/credentials
RUN echo "aws_access_key_id = $AWS_ACCESS_KEY_ID"         >> ~/.aws/credentials
RUN echo "aws_secret_access_key = $AWS_SECRET_ACCESS_KEY" >> ~/.aws/credentials

COPY --chmod=0755 run.sh /
CMD ["/run.sh"]
