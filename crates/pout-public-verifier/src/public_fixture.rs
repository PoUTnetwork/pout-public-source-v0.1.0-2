//! Fixed public synthetic cryptographic verification vectors.
//! No source, seed, trapdoor, proving key or private cache is included.
//! Trusted constants compiled in hash-covered Rust source; this is not a general
//! untrusted verifier-key importer or independent setup ceremony/security review.
use super::{Enrollment, FileRelationError, Response, Verifier};
use crate::AuditContext;
use bellman::groth16;
use bls12_381::Bls12;
use sha2::{Digest, Sha256};
use std::io::Cursor;

pub const VK_HEX: &str = concat!(
    "04afcf1a61bab17b23a9303547f620a426692e019ed99eb1c6f2f6b056563351f56c72e01f84642e147cadde9b80c2dd",
    "0fdfdb400ecdd98c8310c7ca0158b21700ed44053988e4025e4b858b3c9c5acee8fd57093deddca120dea5b1c32ae6ce",
    "060e978fef4fb6617b43d590a69e96daca615b824b277fb72d40cdaf33877ebd81a8469042e3cf4dd00512f8920b9aa9",
    "0a033d312651a4f62d1c0fbb3681a21110bd4657bc29228655a42787b9e9f5d4c6d60d419c9f847b2e6fd8728bb2868d",
    "033b54ed4bf0587108c2864e8eda2adde2c1b5ab59d50f3decd85420bfe83a7730b27e928e98e0bb8c27f61c3e9aa75c",
    "000a7d0a857c2cd8f0daae362ad041f6a47c74ee493da47a95b0c56be2bca3eeaa75bff7343e3e09fb2235cbf9440981",
    "0f0c850d628078a77365c077f0582c3de9a1f17d39ff8908ebba093fb35d8c415157b783cb7c5ef129e478a163016c6a",
    "006a118380db8f9e397f7e5008678f5490abdc4b2df137133b86e2c040c4bc665f55ca1379230351d0c122df8ae6da2f",
    "065b67dcde0bf06fb8090d846d04da655df15d554801392e5e43940a72cff18c86d69186cbc561e864a2a9c29e2d5a73",
    "1578b44d8d88fe3689bdb55631b497724ec604957966c8884d77a1f4c3c8564452d6d70e39556c4cf292a059166d53c8",
    "0c3602f67ed84c8033a666765cd46d876ea624e4dd660cc5784f22197b4305c7a4d4422dcb2d59d943dd2e208558355f",
    "13159bef40247446445becffc8354f711e53b7811a1a9c2d64e23af8340b55ef2ef547021a0d8adc0bba7b34687be353",
    "0933714aabc623cc07eff783a9078ff6780e598faa8f8f75a34d595f3c76c7baa047202d64d7cd04fad2f425d5d8ec86",
    "186a8f303f6aafae8cdbc94670f5a7b67fb878710b871d113793bf1475d5e6263177f83bcf9bd373663a2a3547b2de4b",
    "150c851f5d4921fcdabbe1d3ecd58465945e585cd59187bd483132639d3259076b8db9515418192d0f1d1f6ff090d8cd",
    "08e682efa2cb152d526c2b9bc926491b78374cf71ecd5f4da95e34e1cd609913e9cd1469f6caa8addafbe82e61871965",
    "169ca2246223fbb0dcfa926dea97462ebf0266768ca3119d7f7fc1df7c6e95a653fbebfadb220b4c4a97a458a690cb20",
    "06c8a8c592921ae058eb42cae5f08b83e8e0e297957880915c8818a5ed79b84f88212dd3559b0deb29522e3b70d307c9",
    "0000000608b641ac06a14443ae70c80724635913c09ee150497f8bcde0f01e662e31df3b37737f593dd1708a907936d3",
    "0bfc5a2513c971e9c69bcae407618cd238ea066124e73c54915581c3528e0bbf52c2933b5cb68934d7a0cd9e57dcf2cc",
    "569769c708c1848fa78b1a2e68c4b9490e3db68d990892287a692933413f1300c517878cbf65b6faacc879ea8cdf273d",
    "289833cf0b137857fcfa13c737a5aff9d9297ac1de8f8bbbbb72391c0397798fa7bac8f96ae7094cee51ac4300a03f8b",
    "4f8e02450cf6dc699a4d36267ffc31964c32e1d5a262d44cd7cd4b38d03f48109ddad78f6623259430f7160c6e0ddb73",
    "565bcb02088d72d3c4de07b6f387d2b9d1df3450abe90d3c0b0e0acb86c7a4bbfc663c1706910633ac727f99e346df62",
    "923f232d025bfbc0977e22c19cf6ce9e5b77a1024e09507cd29ced13abe13f81655e03e4894f07e93032ace32cb35b7c",
    "8253f2411720f3ca90d0b74b9e519507719b4fc063746ccc9ad3e1cfd242042535e0457d67e3d9f350f286eff70961f4",
    "e1ab1389024f9d08b6acd406c4c2fe9cbdfd1a8a23115999a1765b7a60604199e0d2ffb60a35c05501cb5f05bd7477e4",
    "e0052510025c7580b20c01c8e202c6d7d599695781759b093e041eea2f6b05707b7fd06365711184cc7bf90a2b19eca9",
    "da8fdc0403fbadf56babbf7d981a8c1824541fa0022e05b1221cc7687f266ed023db0309253652f05a72447045a85ae6",
    "03f5e92d176338ecdb85251cfcd077d2f68e9f2d45b2b4ad6970f435f509364e661d0b280f4c4fc422f9eed126c9e455",
    "d9a0b479",
);
pub const VK_SHA256_HEX: &str = "6cf4d7efdc0cd48bdc295889e24ae538708a5d83121b1a2b514e31f83a8a3c3c";
pub const ENROLLMENT_HEX: &str = concat!(
    "648fde15a302ac465b1ec21bd66c0b3a5ba05d4d045223291cb1d849d81d31824d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d",
    "4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4de9c1dc7572adcd5456e5527664f7e4d5848d2f0610e24efb2d717ee97cad218a",
    "09000000000000005d72cd4af48cba620d83b137fed46bf63e0b6bfc976d249d681022f874e54984",
);
pub const AUDIT0_HEX: &str = concat!(
    "4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d09000000000000000000000000000000",
    "a9a030dad8ba700a6ff6181617dcb670a36eb91366dc85179d2e7b9bcbd3d80f",
);
pub const AUDIT1_HEX: &str = concat!(
    "4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d09000000000000000100000000000000",
    "fe14c44fe804eadb235fe11c41e163595fd3b397fae11cc3c35a521dec24d8d0",
);
pub const TAG0_HEX: &str = "c4d3b94c0bffe1d085352d312d06c7df7e232d57c52ba4f17b8a08c02ace7413";
pub const TAG1_HEX: &str = "6f1f540b064b7cdd101a6f795209219252f3b0f99d6f9876b1791544496384d9";
pub const PROOF0_HEX: &str = concat!(
    "50544652454c30318ab52c27a0156958f1567a4e65eca836cd917384bdce46b70c8a00052b6fa6e46248ffd5884f30a2",
    "7e78b660612dfd96ab392c9b0010db3b34eef98ef42cfa3cf74c79603f8d0d1bd5c6bcef096b13c0b42678bee844a492",
    "bfec23c920744acc0b69656a71cee143ea27c8d105815bfa635041159ccf2cc94789b870d523dcf317faeb208fef91ba",
    "f00351790c8dbf33a3fd2524b363dad7e3b62bedca4d08e8acb4bb5c8a69b312cbbdb5d0b559d71eaa51dcbb72b2d304",
    "f87a530c3bdc1058",
);
pub const PROOF1_HEX: &str = concat!(
    "50544652454c3031b283ea1c2d94255e2babd31233369b4a0ebe16eb2bd1fdfcfc658bf5e7840d5a34518a413b5c5003",
    "581ced454a957a1a87c12776043488368d20e26638a1363edce2ac793f52c2eeed62f96f1756fbdf1c127b322fb33fb6",
    "606b38e9ae25a8c3165f8816a20a536ebf32a6ed124198efab1cd5d5fedd4e119125e66aeea9d2289738ec8826947a26",
    "8252992ef07b6d7bb65faf9fd13bda6813c163c0164b8521cd4fe575998367bf3963a630ef5eab63bc69986f47694d2a",
    "c3727bbbb0ed5517",
);
pub const WRONG_TAG_HEX: &str = "b1a07c1fb3040544e57ad8cd5cf7f894fd3d64edde0ea06892d49d48dd0b220c";
pub const WRONG_PROOF_HEX: &str = concat!(
    "50544652454c3031993ba2849c4fd640e6c4aef7f337851267252d602e86557bbdd14d350ea27d55bc810d4c189e31aa",
    "a74d88943352cf468890c90bb03cdb46a1bb82986c3f8815d1817f6dca1e73351b4a0056d4cfaaa4628269ae25fe48f6",
    "565ce62e9e5161f5098af954161e0cc79ed00eb0aaa1795a32a68a3323e8c37853a8b3cf7225f54c07eb8014074ec9fa",
    "77a75b3ef1ba4e08ad8195d84039bd248f0614242506b5371ceceb421573ebaf73a1796a87f73d7919b97773a9ac1dc8",
    "e339feeea0012649",
);
pub const ALTERNATE_ENROLLMENT_HEX: &str = concat!(
    "ba55fcc7a1910855b1ac176ce41f342ec5dc0fc3686cd7450ae7e772992f475f4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d",
    "4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4de9c1dc7572adcd5456e5527664f7e4d5848d2f0610e24efb2d717ee97cad218a",
    "0900000000000000ad489994b8fa145397746f53c91e26c87a3bae80622b2e404cb225b831e01da5",
);

/// Every field is public data, not a private proof witness.
pub struct PublicFixture {
    pub verifier: Verifier,
    pub delta_g2: [u8; 96],
    pub enrollment: Enrollment,
    pub audits: [AuditContext; 2],
    pub responses: [Response; 2],
    pub wrong_response: Response,
    pub alternate_enrollment: Enrollment,
}
fn fixed<const N: usize>(hex: &str) -> Result<[u8; N], FileRelationError> {
    if hex.len() != N * 2 {
        return Err(FileRelationError::Wire);
    }
    let mut out = [0; N];
    for (out, pair) in out.iter_mut().zip(hex.as_bytes().as_chunks::<2>().0) {
        let digit = |b| match b {
            b'0'..=b'9' => Ok(b - b'0'),
            b'a'..=b'f' => Ok(b - b'a' + 10),
            _ => Err(FileRelationError::Wire),
        };
        *out = digit(pair[0])? * 16 + digit(pair[1])?;
    }
    Ok(out)
}
fn enrollment(bytes: [u8; 136]) -> Enrollment {
    Enrollment {
        source_hash: bytes[..32].try_into().unwrap(),
        network: bytes[32..64].try_into().unwrap(),
        provider: bytes[64..96].try_into().unwrap(),
        epoch: u64::from_le_bytes(bytes[96..104].try_into().unwrap()),
        replica_root: bytes[104..].try_into().unwrap(),
    }
}
/// Load only the fixed trusted constants. Check exact key length/IC count before
/// Bellman's decoder can allocate; pin the complete key before preparation.
pub fn load() -> Result<PublicFixture, FileRelationError> {
    let bytes = fixed::<1444>(VK_HEX)?;
    let pinned = fixed::<32>(VK_SHA256_HEX)?;
    let digest: [u8; 32] = Sha256::digest(bytes).into();
    if bytes[864..868] != 6_u32.to_be_bytes() || digest != pinned {
        return Err(FileRelationError::Wire);
    }
    let mut cursor = Cursor::new(bytes);
    let vk =
        groth16::VerifyingKey::<Bls12>::read(&mut cursor).map_err(|_| FileRelationError::Wire)?;
    if cursor.position() != 1444
        || vk.ic.len() != 6
        || bool::from(vk.alpha_g1.is_identity())
        || bool::from(vk.beta_g1.is_identity())
        || bool::from(vk.beta_g2.is_identity())
        || bool::from(vk.gamma_g2.is_identity())
        || bool::from(vk.delta_g1.is_identity())
        || bool::from(vk.delta_g2.is_identity())
        || vk.ic.iter().any(|point| bool::from(point.is_identity()))
    {
        return Err(FileRelationError::Wire);
    }
    let mut canonical = Vec::with_capacity(1444);
    vk.write(&mut canonical)
        .map_err(|_| FileRelationError::Wire)?;
    if canonical.as_slice() != bytes {
        return Err(FileRelationError::Wire);
    }
    let delta_g2 = vk.delta_g2.to_compressed();
    let verifier = Verifier {
        prepared: groth16::prepare_verifying_key(&vk),
        hash: digest,
        serialized_bytes: 1444,
    };
    Ok(PublicFixture {
        verifier,
        delta_g2,
        enrollment: enrollment(fixed(ENROLLMENT_HEX)?),
        audits: [
            AuditContext::decode(&fixed::<80>(AUDIT0_HEX)?).map_err(|_| FileRelationError::Wire)?,
            AuditContext::decode(&fixed::<80>(AUDIT1_HEX)?).map_err(|_| FileRelationError::Wire)?,
        ],
        responses: [
            Response {
                tag: fixed(TAG0_HEX)?,
                proof: fixed::<200>(PROOF0_HEX)?.to_vec(),
            },
            Response {
                tag: fixed(TAG1_HEX)?,
                proof: fixed::<200>(PROOF1_HEX)?.to_vec(),
            },
        ],
        wrong_response: Response {
            tag: fixed(WRONG_TAG_HEX)?,
            proof: fixed::<200>(WRONG_PROOF_HEX)?.to_vec(),
        },
        alternate_enrollment: enrollment(fixed(ALTERNATE_ENROLLMENT_HEX)?),
    })
}
