{
  validators-commitment-mapper = {
    blob-storage = {
      type = "s3";
      region = "us-east-1";
      bucket-name = "dendreth-validators-commitment-mapper";
      endpoint-url = "http://solunska-server:9000";
      credentials = import
        ./dendreth-validators-commitment-mapper-credentials.nix;
      # credentials = {
      #   access-key-id = "Zgn8GCliFOdwFCkxbAzf";
      #   secret-access-key = "wmOzgxewXtWZMzk6QTvr7Vf0QMYJa1RuDms823TN";
      # };
    };
    metadata-storage = {
      host = "solunska-server";
      port = 6000;
      auth = "";
    };
  };
}
