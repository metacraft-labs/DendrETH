let
  validators-commitment-mapper-credentials = import ./credentials/validators-commitment-mapper.nix;
  balance-verification-credentials = import ./credentials/balance-verification.nix;
in
  {
    validators-commitment-mapper = {
      blob-storage = {
        type = "s3";
        region = "us-east-1";
        bucket-name = "dendreth-validators-commitment-mapper";
        endpoint-url = "http://solunska-server:9000";
        credentials = validators-commitment-mapper-credentials.blob-storage-credentials;
      };
      metadata-storage = {
        host = "solunska-server";
        port = 6000;
        auth = validators-commitment-mapper-credentials.metadata-storage-auth;
      };
    };
    balance-verification = {
      blob-storage = {
        type = "s3";
        region = "us-east-1";
        bucket-name = "dendreth-balance-verification";
        endpoint-url = "http://solunska-server:9000";
        credentials = balance-verification-credentials.blob-storage-credentials;
      };
      metadata-storage = {
        host = "solunska-server";
        port = 6000;
        auth = balance-verification-credentials.metadata-storage-auth;
      };
    };
  }
