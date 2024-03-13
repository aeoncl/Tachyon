pub struct CvrRequest {
    pub tr_id: u128,
    pub first_candidate : MsnpVersion,
    pub second_candidate : MsnpVersion,
    pub cvr: String
}