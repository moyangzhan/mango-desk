use crate::entities::ModelPlatform;
use crate::structs::proxy_setting::ProxyInfo;
pub trait WithPlatformConfig {
    fn platform(&self) -> &ModelPlatform;
    fn proxy(&self) -> &ProxyInfo;
}
