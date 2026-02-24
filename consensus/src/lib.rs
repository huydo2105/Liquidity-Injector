pub mod validator;
pub mod collector;

pub mod quorum_proto {
    tonic::include_proto!("quorum");
}
