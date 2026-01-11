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

    #[derive(Clone, Debug)]    
    pub enum ExternalRun {
        $(
            $backend($run)
        ),*
    }

    #[derive(Clone, Debug)]
    pub enum ExternalSong {
        $(
            $backend($song)
        ),*
    }

    #[derive(Clone, Debug)]
    pub enum ExternalType {
        $(
            $backend
        ),*
    }

    }
}