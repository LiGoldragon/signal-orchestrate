//! Authority-seated identities for the strict coordination Interface.
//!
//! These opaque identities and canonical-order values are minted state. None
//! is derived from spelling, source position, or Rust representation.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthoritySeat {
    pub spelling: &'static str,
    pub local: u16,
    pub canonical: u64,
}

impl AuthoritySeat {
    pub const fn new(spelling: &'static str, local: u16, canonical: u64) -> Self {
        Self {
            spelling,
            local,
            canonical,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeclarationSeat {
    pub owner_local: Option<u16>,
    pub spelling: &'static str,
    pub local: u16,
    pub canonical: u64,
}

impl DeclarationSeat {
    pub const fn new(
        owner_local: Option<u16>,
        spelling: &'static str,
        local: u16,
        canonical: u64,
    ) -> Self {
        Self {
            owner_local,
            spelling,
            local,
            canonical,
        }
    }
}

pub const AUTHORITY_IDENTITY: [u8; 32] = [
    68, 230, 144, 32, 240, 2, 227, 106, 51, 1, 105, 49, 52, 2, 96, 70, 65, 181, 17, 220, 32, 165,
    226, 191, 14, 26, 243, 97, 175, 4, 105, 92,
];
pub const AUTHORITY_REVISION: u64 = 1;
pub const GRAMMAR_DOCUMENT_LOCAL: u16 = 38507;
pub const GRAMMAR_SYNTAX_LOCAL: u16 = 44879;

pub const INTERFACE_SEAT: AuthoritySeat = AuthoritySeat::new("Interface", 2339, 0x69de7fd166248bf9);
pub const NEXUS_SEAT: AuthoritySeat = AuthoritySeat::new("Nexus", 3235, 0x331959aa1e843299);
pub const SEMA_SEAT: AuthoritySeat = AuthoritySeat::new("Sema", 29501, 0xe2a4350c99ae55dd);
pub const INPUT_SEAT: AuthoritySeat = AuthoritySeat::new("Input", 8732, 0x8894b5ef0707c2a2);
pub const OUTPUT_SEAT: AuthoritySeat = AuthoritySeat::new("Output", 24382, 0xcb74b215d6f48caf);
pub const REFUSAL_SEAT: AuthoritySeat = AuthoritySeat::new("Refusal", 41050, 0x5845b33dbf737f51);
pub const STRING_SEAT: AuthoritySeat = AuthoritySeat::new("String", 22764, 0x40799e630ab7d6fe);
pub const INTEGER_SEAT: AuthoritySeat = AuthoritySeat::new("Integer", 37512, 0x4d7a0baceb981840);
pub const BOOLEAN_SEAT: AuthoritySeat = AuthoritySeat::new("Boolean", 5827, 0xa8b18a6031e7376e);
pub const UNIT_SEAT: AuthoritySeat = AuthoritySeat::new("Unit", 57783, 0x99dee585f7a5ed28);
pub const VECTOR_SEAT: AuthoritySeat = AuthoritySeat::new("Vector", 54192, 0x72811aca319e44ad);
pub const OPTION_SEAT: AuthoritySeat = AuthoritySeat::new("Option", 53590, 0xfca9637601fb9618);
pub const MAP_SEAT: AuthoritySeat = AuthoritySeat::new("Map", 27798, 0x552f219cb39f22ba);
pub const RESULT_SEAT: AuthoritySeat = AuthoritySeat::new("Result", 28566, 0xf297d74464de032a);
pub const STREAM_SEAT: AuthoritySeat = AuthoritySeat::new("Stream", 35711, 0x6d525d7c5ec015fa);
pub const STREAM_IDENTITY_SEAT: AuthoritySeat =
    AuthoritySeat::new("StreamIdentity", 49810, 0x6de2d2bfea0bd258);

pub const RUST_VOCABULARY_LOCALS: [u16; 10] = [
    33198, 30525, 58862, 40508, 28089, 48079, 45548, 58097, 50465, 41343,
];

pub const DECLARATION_SEATS: &[DeclarationSeat] = &[
    DeclarationSeat::new(None, "RoleIdentifier", 18813, 0xc18dc23f2855c622),
    DeclarationSeat::new(None, "RoleToken", 1120, 0x24f34f663a0b27fc),
    DeclarationSeat::new(None, "RoleTokens", 45989, 0x655eb2f25d73d986),
    DeclarationSeat::new(None, "Role", 2890, 0xc28e88d406d76a18),
    DeclarationSeat::new(None, "SessionIdentifier", 50058, 0x24f80168fb9c28cc),
    DeclarationSeat::new(None, "LaneAuthority", 35255, 0x22678e98dac598dc),
    DeclarationSeat::new(Some(35255), "Structural", 10352, 0x1246e699db97d798),
    DeclarationSeat::new(Some(35255), "Support", 19874, 0x02aceb280b52b392),
    DeclarationSeat::new(None, "LaneIdentifier", 54152, 0x5eee72f3aa5e4483),
    DeclarationSeat::new(None, "LaneDetails", 13910, 0xe9a6e5cf89716671),
    DeclarationSeat::new(None, "LaneStatus", 38511, 0xccf157a5dcfc538e),
    DeclarationSeat::new(Some(38511), "Active", 25909, 0xdbdcb29e05359a83),
    DeclarationSeat::new(Some(38511), "Released", 37919, 0x7dd6c1fa851f3b36),
    DeclarationSeat::new(Some(38511), "HandoverEnded", 19557, 0x26152ca411e24183),
    DeclarationSeat::new(Some(38511), "Suspect", 29745, 0xa9f10bba5c4d9da5),
    DeclarationSeat::new(None, "LaneOwner", 60928, 0x2871564b95671aee),
    DeclarationSeat::new(None, "LaneAssignment", 27344, 0xde460fb2861a4046),
    DeclarationSeat::new(None, "WirePath", 17604, 0xc8bb6282ea3ae316),
    DeclarationSeat::new(None, "TaskToken", 35445, 0x28a88086be60cd5d),
    DeclarationSeat::new(None, "ScopeReason", 54049, 0xa3abd67cebe4e2e5),
    DeclarationSeat::new(None, "ScopeReference", 19965, 0xbaf34e66a9fcbde8),
    DeclarationSeat::new(Some(19965), "Path", 35878, 0xdcbc5e6bb33972d9),
    DeclarationSeat::new(Some(19965), "Task", 2475, 0x747f183c297ae485),
    DeclarationSeat::new(None, "TimestampNanos", 33829, 0x98258b7179bcb42a),
    DeclarationSeat::new(None, "DurationNanos", 22250, 0x27627ac30a599eb6),
    DeclarationSeat::new(None, "ScopeReferences", 62109, 0xcf31d6cad52338f0),
    DeclarationSeat::new(None, "ScopeConflict", 14058, 0xacf9a8b13d2ebb57),
    DeclarationSeat::new(None, "ScopeConflicts", 70, 0xc9796959fff6cec8),
    DeclarationSeat::new(None, "LaneResourceClaim", 5646, 0xa813727d0c357838),
    DeclarationSeat::new(None, "LaneResourceClaims", 40524, 0x115865840403164e),
    DeclarationSeat::new(None, "RepositoryName", 41447, 0x6dff24b045bdffed),
    DeclarationSeat::new(None, "BranchName", 60207, 0x342fc257ac2768d6),
    DeclarationSeat::new(None, "LaneName", 26299, 0x83da2699de7f2eae),
    DeclarationSeat::new(None, "PurposeText", 26939, 0xf7d41a35f53eaf27),
    DeclarationSeat::new(None, "WorktreeStatus", 53683, 0xc3d3284888cac593),
    DeclarationSeat::new(Some(53683), "Active", 52230, 0xefdcd93a051bd69a),
    DeclarationSeat::new(Some(53683), "Merged", 9150, 0x3b1f5f38ac8cc479),
    DeclarationSeat::new(Some(53683), "Archived", 31701, 0xf31c5975e2a57aa3),
    DeclarationSeat::new(Some(53683), "Recycled", 31145, 0xd0145e04c52f409c),
    DeclarationSeat::new(Some(53683), "Abandoned", 9520, 0x94b5b05c95b1cfb2),
    DeclarationSeat::new(None, "PushedState", 38542, 0xd34dfd9f1d2dddad),
    DeclarationSeat::new(Some(38542), "Unpushed", 12630, 0x0167307e349f1268),
    DeclarationSeat::new(Some(38542), "Pushed", 36160, 0x687fbfadcfd8b2e8),
    DeclarationSeat::new(Some(38542), "AncestorOfMain", 33840, 0x878eece4e9153d6d),
    DeclarationSeat::new(None, "Worktree", 54130, 0x212a7e0a28e80d97),
    DeclarationSeat::new(None, "Worktrees", 64126, 0xaedfd8c299c7904c),
    DeclarationSeat::new(None, "RoleClaim", 28689, 0x6a5d10a302453dfe),
    DeclarationSeat::new(None, "RoleRelease", 5325, 0x044f7344915ad4fd),
    DeclarationSeat::new(None, "ClaimAcceptance", 41300, 0xc5183ad723f0ec2e),
    DeclarationSeat::new(None, "ClaimRejection", 39724, 0xeac6ff03ecd92438),
    DeclarationSeat::new(None, "ReleaseAcknowledgment", 54599, 0xf4d1978e340ba0b9),
];
