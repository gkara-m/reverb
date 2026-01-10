#[macro_export]
macro_rules! external_types {
    (
        $(
            $backend:ident {
            Run:  $run:ty,
            Song: $song:ty $(,)?
        }
        ),* $(,)?
    ) => {

    pub enum ExternalRun {
        $(
            $backend($run)
        ),*
    }

    pub enum ExternalSong {
        $(
            $backend($song)
        ),*
    }

    pub enum ExternalType {
        $(
            $backend
        ),*
    }

    }
}