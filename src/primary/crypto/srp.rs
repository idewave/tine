use num_bigint::{BigInt, Sign};
use num_traits::{FromPrimitive, Num};
use sha1::{Digest};

const MODULUS: &str = "894B645E89E1535BBDAD5B8B290650530801B18EBFBF5E8FAB3C82872A3E9BB7";

#[derive(Debug)]
pub struct Srp {
    pub modulus: BigInt,
    pub generator: BigInt,
    pub salt: [u8; 32],
    pub server_ephemeral: Option<BigInt>,
    pub session_key: Option<Vec<u8>>,
    pub multiplier: BigInt,
    private_ephemeral: BigInt,
    account: Option<String>,
    verifier: Option<BigInt>
}

// public methods
impl Srp {
    pub fn new() -> Self {
        let modulus = BigInt::from_str_radix(MODULUS, 16).unwrap();
        let generator = BigInt::from_i32(7).unwrap();
        let private_ephemeral = Self::generate_private_ephemeral();

        Self {
            modulus,
            generator,
            salt: rand::random(),
            server_ephemeral: None,
            session_key: None,
            multiplier: BigInt::from_i32(3).unwrap(),
            private_ephemeral,
            account: None,
            verifier: None,
        }
    }

    pub fn calculate_proof<D>(&mut self, client_ephemeral: &[u8]) -> Vec<u8> where D: Digest {
        let server_ephemeral = self.server_ephemeral.as_mut().unwrap().clone();
        let session_key = self.session_key.as_ref().unwrap().to_vec();

        D::new()
            .chain(self.calculate_xor_hash::<D>())
            .chain(self.calculate_account_hash::<D>())
            .chain(self.salt)
            .chain(client_ephemeral)
            .chain(server_ephemeral.to_bytes_le().1)
            .chain(session_key)
            .finalize()
            .to_vec()
    }

    pub fn generate_verifier<D>(&mut self) where D: Digest {
        let x = self.calculate_x::<D>();
        let verifier = self.generator.modpow(
            &x,
            &self.modulus,
        );
        self.verifier = Some(verifier);
    }

    pub fn generate_server_ephemeral<D>(&mut self) where D: Digest {
        let v = self.verifier.as_ref().unwrap();
        let big_integer = self.generator.modpow(&self.private_ephemeral, &self.modulus);
        self.server_ephemeral = Some((&self.multiplier * v + &big_integer) % &self.modulus);
    }

    pub fn calculate_session_key<D>(&mut self, client_ephemeral: &[u8]) where D: Digest {
        let v = { self.verifier.as_ref().unwrap().clone() };
        let u = self.calculate_u::<D>(client_ephemeral);
        let a_pub_num = BigInt::from_bytes_le(Sign::Plus, client_ephemeral);
        let s = {
            let s = (v.modpow(&u, &self.modulus) * a_pub_num)
                .modpow(&self.private_ephemeral, &self.modulus);
            BigInt::from_biguint(Sign::Plus, s.to_biguint().unwrap())
        };

        self.session_key = Some(Self::calculate_interleaved::<D>(s));
    }

    pub fn set_account(&mut self, account: String) {
        self.account = Some(account.to_uppercase());
    }
}

// private methods
impl Srp {
    fn calculate_account_hash<D>(&mut self) -> Vec<u8>
        where
            D: Digest
    {
        let account = self.account.as_ref().unwrap();
        D::new()
            .chain(account.as_bytes())
            .finalize()
            .to_vec()
    }

    fn calculate_xor_hash<D>(&mut self) -> Vec<u8>
        where
            D: Digest,
    {
        let n_hash = D::new().chain(self.modulus.to_bytes_le().1).finalize();
        let g_hash = D::new().chain(self.generator.to_bytes_le().1).finalize();

        let mut xor_hash = Vec::new();
        for (index, value) in g_hash.iter().enumerate() {
            xor_hash.push(value ^ n_hash[index]);
        }

        xor_hash
    }

    fn calculate_x<D>(&mut self) -> BigInt
        where
            D: Digest,
    {
        let account = self.account.as_ref().unwrap();
        let identity_hash = D::new()
            .chain(format!("{}:{}", account, account).as_bytes())
            .finalize()
            .to_vec();

        let x = D::new()
            .chain(self.salt)
            .chain(identity_hash)
            .finalize()
            .to_vec();

        BigInt::from_bytes_le(Sign::Plus, &x)
    }

    fn calculate_u<D>(&mut self, client_ephemeral: &[u8]) -> BigInt
        where
            D: Digest,
    {
        let server_ephemeral = self.server_ephemeral.as_ref().unwrap();
        let u = D::new()
            .chain(client_ephemeral)
            .chain(server_ephemeral.to_bytes_le().1)
            .finalize()
            .to_vec();

        BigInt::from_bytes_le(Sign::Plus, &u)
    }

    fn calculate_interleaved<D>(s: BigInt) -> Vec<u8>
        where
            D: Digest
    {
        let (even, odd): (Vec<_>, Vec<_>) =
            s.to_bytes_le().1
                .into_iter()
                .enumerate()
                .partition(|(i, _)| i % 2 == 0);

        let part1 = even.iter().map(|(_, v)| *v).collect::<Vec<u8>>();
        let part2 = odd.iter().map(|(_, v)| *v).collect::<Vec<u8>>();

        let hashed1 = D::new().chain(part1).finalize();
        let hashed2 = D::new().chain(part2).finalize();

        let mut session_key = Vec::new();
        for (index, _) in hashed1.iter().enumerate() {
            session_key.push(hashed1[index]);
            session_key.push(hashed2[index]);
        }

        session_key
    }

    fn generate_private_ephemeral() -> BigInt {
        let random_bytes: [u8; 19] = rand::random();
        BigInt::from_bytes_le(Sign::Plus, &random_bytes)
    }
}