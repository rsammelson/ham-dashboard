use super::{RSTError, RST};

#[test]
fn too_short() {
    assert!(matches!(
        RST::try_from("5"),
        Err(RSTError::InvalidLength(_))
    ));
}

#[test]
fn too_long() {
    assert!(matches!(
        RST::try_from("1111"),
        Err(RSTError::InvalidLength(_))
    ));
}

#[test]
fn invalid_char() {
    assert!(matches!(
        RST::try_from("11a"),
        Err(RSTError::InvalidCharacter(_))
    ));
}

#[test]
fn digit_too_small() {
    assert!(matches!(
        RST::try_from("110"),
        Err(RSTError::ValueTooSmall(_))
    ));
}

#[test]
fn digit_too_large() {
    assert!(matches!(
        RST::try_from("611"),
        Err(RSTError::ValueTooLarge(..))
    ));
}

#[test]
fn good() {
    assert_eq!(RST::try_from("11").unwrap(), RST::new(1, 1, None).unwrap());
    assert_eq!(
        RST::try_from("111").unwrap(),
        RST::new(1, 1, Some(1)).unwrap()
    );
    assert_eq!(
        RST::try_from("599").unwrap(),
        RST::new(5, 9, Some(9)).unwrap()
    );
}

#[test]
fn new_good() {
    let report1 = RST::new(1, 1, None).unwrap();
    assert_eq!(report1.readability(), 1);
    assert_eq!(report1.strength(), 1);
    assert_eq!(report1.tone(), None);

    let report2 = RST::new(1, 1, Some(1)).unwrap();
    assert_eq!(report2.readability(), 1);
    assert_eq!(report2.strength(), 1);
    assert_eq!(report2.tone(), Some(1));

    let report3 = RST::new(5, 9, Some(9)).unwrap();
    assert_eq!(report3.readability(), 5);
    assert_eq!(report3.strength(), 9);
    assert_eq!(report3.tone(), Some(9));
}

#[test]
fn new_bad() {
    assert!(matches!(
        RST::new(0, 1, None),
        Err(RSTError::ValueTooSmall(_))
    ));
    assert!(matches!(
        RST::new(1, 0, Some(1)),
        Err(RSTError::ValueTooSmall(_))
    ));
    assert!(matches!(
        RST::new(1, 1, Some(0)),
        Err(RSTError::ValueTooSmall(_))
    ));

    assert!(matches!(
        RST::new(6, 1, None),
        Err(RSTError::ValueTooLarge(..))
    ));
    assert!(matches!(
        RST::new(1, 10, Some(1)),
        Err(RSTError::ValueTooLarge(..))
    ));
    assert!(matches!(
        RST::new(1, 1, Some(10)),
        Err(RSTError::ValueTooLarge(..))
    ));
}

#[test]
fn display() {
    assert_eq!(format!("{}", RST::try_from("11").unwrap()), "11");
    assert_eq!(format!("{}", RST::try_from("111").unwrap()), "111");
    assert_eq!(format!("{}", RST::try_from("599").unwrap()), "599");
}
