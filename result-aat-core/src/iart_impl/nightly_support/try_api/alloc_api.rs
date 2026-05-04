#![doc = include_str!("../../../../doc/modules/alloc_api.md")]

use crate::types::Iart;

use crate::events::{AutoRequestType, IartEvent};
use crate::IartErr;
use core::convert::Infallible;
use core::fmt::Debug;
use core::ops::{ControlFlow, FromResidual, Try};

impl<
        Item: Debug + 'static,
        A: alloc::alloc::Allocator + Clone + 'static + Default + Send + Sync + Debug,
    > Try for Iart<Item, A>
{
    type Output = Item;
    type Residual = Iart<Infallible, A>;

    #[inline]
    fn from_output(output: Self::Output) -> Iart<Item, A> {
        Iart::<Item, A>::new_ok_in(output, A::default())
    }

    #[track_caller]
    #[doc = include_str!("../../../../doc/fn/Iart/branch.md")]
    fn branch(mut self) -> ControlFlow<Self::Residual, Self::Output> {
        self.send_log();

        let _ = unsafe {
            self.send_log_to_handler::<true>(IartEvent::FunctionHook(AutoRequestType::TryUsed))
                .unwrap_unchecked()
        };

        match self.data.take() {
            Some(Ok(_)) => {
                self.handled = true;
                self.data = Some(Ok(()));
                ControlFlow::Continue(self.item.take().unwrap())
            }
            Some(Err(err)) => {
                self.data = Some(Err(err));
                ControlFlow::Break(self.map(|_| unreachable!()))
            }
            None => panic!("Iart: try branch called after consumption"),
        }
    }
}

impl<
        Item: 'static,
        A: alloc::alloc::Allocator + Clone + 'static + Default + Send + Sync + Debug,
    > FromResidual<Iart<Infallible, A>> for Iart<Item, A>
{
    #[track_caller]
    fn from_residual(mut residual: Iart<Infallible, A>) -> Self {
        residual.send_log();

        residual.internal_map(|_| unreachable!())
    }
}

impl<Item: 'static, A> core::ops::Residual<Item> for Iart<Infallible, A>
where
    A: alloc::alloc::Allocator + Clone + 'static + Default + Send + Sync + Debug,
    Item: Debug,
{
    type TryType = Iart<Item, A>;
}

impl<Item: 'static, E, A> FromResidual<Result<Infallible, E>> for Iart<Item, A>
where
    E: IartErr<A> + Send + Sync + 'static,
    A: alloc::alloc::Allocator + Clone + 'static + Default + Debug,
{
    #[track_caller]
    fn from_residual(residual: Result<Infallible, E>) -> Self {
        let err = unsafe { residual.unwrap_err_unchecked() };

        Self::new_err_in(err, None, A::default())
    }
}
