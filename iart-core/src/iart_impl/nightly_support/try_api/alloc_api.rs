use crate::types::Iart;

use crate::IartErr;
use crate::events::{AutoRequestType, IartEvent};
use core::convert::Infallible;
use core::fmt::Debug;
use core::ops::{ControlFlow, FromResidual, Try};

impl<Item: Debug, A: alloc::alloc::Allocator + Clone + 'static + Default> Try for Iart<Item, A> {
    type Output = Item;
    type Residual = Iart<Infallible, A>;

    #[inline]
    fn from_output(output: Self::Output) -> Iart<Item, A> {
        Iart::<Item, A>::Ok_in(output, A::default())
    }

    #[track_caller]
    fn branch(mut self) -> ControlFlow<Self::Residual, Self::Output> {
        self.send_log();

        let _ = unsafe {
            self.send_log_to_handler::<true>(IartEvent::FunctionHook(AutoRequestType::TryUsed))
                .unwrap_unchecked()
        };

        match self.data.take() {
            Some(Ok(item)) => {
                self.handled = true;
                ControlFlow::Continue(item)
            }
            Some(Err(err)) => {
                let res: Iart<Infallible, A> = self.map();
            }
            None => panic!("Iart: try branch called after consumption"),
        }
    }
}

impl<Item, A: alloc::alloc::Allocator + Clone + 'static + Default> FromResidual<Iart<Infallible, A>>
    for Iart<Item, A>
{
    #[track_caller]
    fn from_residual(mut residual: Iart<Infallible, A>) -> Self {
        let alloc = residual.allocator.clone();
        residual.handled = true;
        Self {
            data: residual
                .data
                .take()
                .map(|d| Err(unsafe { d.unwrap_err_unchecked() })),
            handled: false,
            #[cfg(feature = "allow-backtrace-logging")]
            log: residual.log.take(),
            allocator: alloc,
            #[cfg(feature = "error-can-have-item")]
            err_item: residual.err_item.take(),
            trans_fns: residual.trans_fns,
        }
    }
}

impl<Item, A> core::ops::Residual<Item> for Iart<Infallible, A>
where
    A: alloc::alloc::Allocator + Clone + 'static + Default,
    Item: Debug,
{
    type TryType = Iart<Item, A>;
}

impl<Item, E, A> FromResidual<Result<Infallible, E>> for Iart<Item, A>
where
    E: IartErr<A> + Send + Sync + 'static,
    A: alloc::alloc::Allocator + Clone + 'static + Default,
{
    #[track_caller]
    fn from_residual(residual: Result<Infallible, E>) -> Self {
        let err = unsafe { residual.unwrap_err_unchecked() };

        Self::Err_in(err, None, A::default())
    }
}
