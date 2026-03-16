use crate::entities::SelfHostedPlatform;

pub trait WithSelfHostedConfig {
    fn platform(&self) -> &SelfHostedPlatform;
    fn base_url(&self) -> String {
        format!("http://{}:{}", self.platform().host, self.platform().port)
    }
}
