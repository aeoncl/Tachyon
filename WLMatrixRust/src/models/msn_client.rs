use std::{sync::{RwLock, Arc, RwLockWriteGuard, atomic::{AtomicI16, Ordering}}};

use crate::repositories::switchboard_repository::SwitchboardRepository;

use super::msn_user::MSNUser;


#[derive(Clone)]

pub struct MSNClient {
    pub(crate) inner: Arc<MSNClientInner>,
}

pub(crate) struct MSNClientInner {
    user: RwLock<MSNUser>,
    msnp_version: AtomicI16,
    switchboards : SwitchboardRepository
}

impl MSNClient {
    pub fn new(user: MSNUser, msnp_version: i16) -> Self {

        let inner = Arc::new(MSNClientInner {
            user: RwLock::new(user),
            msnp_version: AtomicI16::new(msnp_version),
            switchboards: SwitchboardRepository::new(),
        });

        return MSNClient { inner };
    }

    pub fn get_user(&self) -> MSNUser {
        return self.inner.user.read().unwrap().clone();
    }

    pub fn get_user_msn_addr(&self) -> String {
       return self.inner.user.read().unwrap().get_msn_addr();
    }

    pub fn get_user_mut(&mut self) -> RwLockWriteGuard<MSNUser> {
        return self.inner.user.write().unwrap();
    }

    pub fn set_msnp_version(&mut self, msnp_version: i16){
        self.inner.msnp_version.store(msnp_version, Ordering::SeqCst);
    }

    pub fn get_msnp_version(&self) -> i16 {
        return self.inner.msnp_version.load(Ordering::SeqCst);
    }

    pub fn get_switchboards(&self) -> &SwitchboardRepository{
        return &self.inner.switchboards;
    }

}

