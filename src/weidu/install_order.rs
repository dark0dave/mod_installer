use std::{error::Error, slice::Iter};

use crate::{weidu::batched_components::WeiduBatchedComponents, weidu::component::WeiduComponent};

#[derive(Debug, PartialEq)]
pub(crate) struct WeiduBatchedInstallOrder(Vec<WeiduBatchedComponents>);

impl WeiduBatchedInstallOrder {
  pub(crate) fn new(components: WeiduBatchedComponents) -> Self {
    Self(vec![components])
  }
  pub(crate) fn batch(components: WeiduBatchedComponents) -> Result<Self, Box<dyn Error>> {
    let mut out: Vec<WeiduBatchedComponents> = vec![];
    for component in components.into_iter() {
      match out.last_mut() {
        Some(current)
          if current.last().unwrap_or(&WeiduComponent::default()).tp_file == component.tp_file =>
        {
          current.push(component.clone());
        },
        _ => out.push(vec![component.clone()].into()),
      }
    }
    Ok(Self(out))
  }
}

impl<'a> IntoIterator for &'a WeiduBatchedInstallOrder {
  type Item = &'a WeiduBatchedComponents;
  type IntoIter = Iter<'a, WeiduBatchedComponents>;

  fn into_iter(self) -> Self::IntoIter {
    self.0.iter()
  }
}

#[cfg(test)]
mod tests {

  use std::path::PathBuf;

  use super::*;
  use pretty_assertions::assert_eq;

  #[test]
  fn test_batching() -> Result<(), Box<dyn Error>> {
    let weidu_fixture_path = PathBuf::from("fixtures/test_batching.log");
    let weidu_log_file = WeiduBatchedComponents::try_from(weidu_fixture_path)?;
    let result = WeiduBatchedInstallOrder::batch(weidu_log_file)?;
    let expected = WeiduBatchedInstallOrder(vec![
      vec![WeiduComponent {
        tp_file: "TEST.TP2".into(),
        name: "TEST_MOD_NAME_1".into(),
        lang: "0".into(),
        component: "0".into(),
        component_name: "test mod one".into(),
        sub_component: "".into(),
        version: "".into(),
      }]
      .into(),
      vec![WeiduComponent {
        tp_file: "Portraits.TP2".into(),
        name: "TEST_MOD_NAME_5".into(),
        lang: "0".into(),
        component: "8".into(),
        component_name: "Add Baddies Portraits".into(),
        sub_component: "".into(),
        version: "".into(),
      }]
      .into(),
      vec![WeiduComponent {
        tp_file: "TEST.TP2".into(),
        name: "TEST_MOD_NAME_1".into(),
        lang: "0".into(),
        component: "1".into(),
        component_name: "test mod two".into(),
        sub_component: "".into(),
        version: "".into(),
      }]
      .into(),
      vec![
        WeiduComponent {
          tp_file: "END.TP2".into(),
          name: "TEST_MOD_NAME_2".into(),
          lang: "0".into(),
          component: "0".into(),
          component_name: "test mod with subcomponent information".into(),
          sub_component: "Standard installation".into(),
          version: "".into(),
        },
        WeiduComponent {
          tp_file: "END.TP2".into(),
          name: "TEST_MOD_NAME_3".into(),
          lang: "0".into(),
          component: "0".into(),
          component_name: "test mod with version".into(),
          sub_component: "".into(),
          version: "1.02".into(),
        },
      ]
      .into(),
      vec![WeiduComponent {
        tp_file: "TWEAKS.TP2".into(),
        name: "TEST_MOD_NAME_4".into(),
        lang: "0".into(),
        component: "3346".into(),
        component_name: "test mod with both subcomponent information and version".into(),
        sub_component: "Casting speed only".into(),
        version: "v16".into(),
      }]
      .into(),
      vec![
        WeiduComponent {
          tp_file: "Portraits.TP2".into(),
          name: "TEST_MOD_NAME_5".into(),
          lang: "0".into(),
          component: "3346".into(),
          component_name: "Add Portraits".into(),
          sub_component: "".into(),
          version: "".into(),
        },
        WeiduComponent {
          tp_file: "Portraits.TP2".into(),
          name: "TEST_MOD_NAME_5".into(),
          lang: "0".into(),
          component: "3".into(),
          component_name: "Add Baddies Portraits".into(),
          sub_component: "".into(),
          version: "".into(),
        },
        WeiduComponent {
          tp_file: "Portraits.TP2".into(),
          name: "TEST_MOD_NAME_5".into(),
          lang: "0".into(),
          component: "7".into(),
          component_name: "Add GoodiwaPortraits".into(),
          sub_component: "".into(),
          version: "".into(),
        },
      ]
      .into(),
    ]);
    assert_eq!(result, expected);
    Ok(())
  }
}
