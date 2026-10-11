use napi::Either;

/// The policies that restrict which requesters may use a feature.
#[napi(object)]
pub struct Policies {
  pub requester: RequesterPolicy,
}

/// Either only the `allow`ed requesters may use the feature,
/// or every requester except the `block`ed ones may use it.
#[napi]
pub type RequesterPolicy = Either<AllowRequesterPolicy, BlockRequesterPolicy>;

/// Only the listed requesters may use the feature.
#[napi(object)]
pub struct AllowRequesterPolicy {
  pub allow: Vec<String>,
}

/// Every requester except the listed ones may use the feature.
#[napi(object)]
pub struct BlockRequesterPolicy {
  pub block: Vec<String>,
}

impl From<optify::Policies> for Policies {
  fn from(policies: optify::Policies) -> Self {
    Self {
      requester: match policies.requester {
        optify::RequesterPolicy::Allow { allow } => Either::A(AllowRequesterPolicy {
          allow: allow.into_iter().collect(),
        }),
        optify::RequesterPolicy::Block { block } => Either::B(BlockRequesterPolicy {
          block: block.into_iter().collect(),
        }),
      },
    }
  }
}
