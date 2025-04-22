use httpmock::MockServer;
use ipfs::IpfsClient;

pub fn setup_ipfs_mock() -> (MockServer, IpfsClient) {
    let server = MockServer::start();
    let ipfs_client = IpfsClient::from_url(&format!("{}/ipfs/", server.base_url()));

    (server, ipfs_client)
}
