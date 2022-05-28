use crate::Bind;

/// Trait for tree-like structures that can be recursively transformed with
/// effectful computations.
pub trait TreeWalk<FS>: Bind<FS> {
    /// Applies an effectful function to each branch of the tree, wrapping the
    /// final value in the same type of effect.
    fn each_branch(self, f: impl FnMut(Self) -> FS) -> FS;

    /// Applies an effectful function to every node of a tree, including the
    /// root itself, in a bottom-up manner.
    fn bottom_up(self, mut f: impl FnMut(Self) -> FS) -> FS {
        fn go<S: TreeWalk<FS>, FS>(
            branch: S,
            f: &mut impl FnMut(S) -> FS,
        ) -> FS {
            let rest_transformed = branch.each_branch(|branch| go(branch, f));
            Bind::bind_mut(rest_transformed, f)
        }
        go(self, &mut f)
    }
}
