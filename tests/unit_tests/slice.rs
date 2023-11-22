use paradis_core::{IntoUnsyncAccess, UnsyncAccess};

#[test]
fn test_basic_access() {
    let slice = &mut [0, 1, 2, 3];
    let access = slice.into_unsync_access();

    assert_eq!(access.len(), 4);
    assert_eq!(unsafe { access.get_unsync(0) }, &0);
    assert_eq!(unsafe { access.get_unsync(1) }, &1);
    assert_eq!(unsafe { access.get_unsync(2) }, &2);
    assert_eq!(unsafe { access.get_unsync(3) }, &3);
    assert_eq!(unsafe { access.get_unsync_mut(0) }, &0);
    assert_eq!(unsafe { access.get_unsync_mut(1) }, &1);
    assert_eq!(unsafe { access.get_unsync_mut(2) }, &2);
    assert_eq!(unsafe { access.get_unsync_mut(3) }, &3);

    let access2 = unsafe { access.clone_access() };
    assert_eq!(access2.len(), 4);
    assert_eq!(unsafe { access2.get_unsync(0) }, &0);
    assert_eq!(unsafe { access2.get_unsync(1) }, &1);
    assert_eq!(unsafe { access2.get_unsync(2) }, &2);
    assert_eq!(unsafe { access2.get_unsync(3) }, &3);
    assert_eq!(unsafe { access2.get_unsync_mut(0) }, &0);
    assert_eq!(unsafe { access2.get_unsync_mut(1) }, &1);
    assert_eq!(unsafe { access2.get_unsync_mut(2) }, &2);
    assert_eq!(unsafe { access2.get_unsync_mut(3) }, &3);

    // Obtain mutable references to non-overlapping entries from two different accesses.
    {
        let a: &mut u32 = unsafe { access.get_unsync_mut(0) };
        let b: &mut u32 = unsafe { access2.get_unsync_mut(1) };
        let c: &mut u32 = unsafe { access.get_unsync_mut(2) };
        let d: &mut u32 = unsafe { access2.get_unsync_mut(3) };

        *a = 4;
        *b = 5;
        *c = 6;
        *d = 7;

        assert_eq!(unsafe { access2.get_unsync(0) }, &4);
        assert_eq!(unsafe { access.get_unsync(1) }, &5);
        assert_eq!(unsafe { access2.get_unsync(2) }, &6);
        assert_eq!(unsafe { access.get_unsync(3) }, &7);
    }

    // Obtain overlapping immutable references. We run this through miri to check that
    // this does not cause any unexpected issues
    {
        let a1: &u32 = unsafe { access.get_unsync(0) };
        let a2: &u32 = unsafe { access.get_unsync(0) };
        let a3: &u32 = unsafe { access2.get_unsync(0) };
        let a4: &u32 = unsafe { access2.get_unsync(0) };
        let b1: &u32 = unsafe { access.get_unsync(1) };
        let b2: &u32 = unsafe { access.get_unsync(1) };
        let b3: &u32 = unsafe { access2.get_unsync(1) };
        let b4: &u32 = unsafe { access2.get_unsync(1) };
        assert_eq!(a1, a2);
        assert_eq!(a2, a3);
        assert_eq!(a3, a4);
        assert_eq!(b1, b2);
        assert_eq!(b2, b3);
        assert_eq!(b3, b4);
    }
}
