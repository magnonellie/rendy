
use escape::Terminal;

/// Global resource tracker.
/// This object catches dropped resources
/// and ensures that they aren't used by device before actually destroying them.
/// It can preserve a resource for longer time than needed
/// but never destroys resource before device stops using it.
pub struct GlobalTracker(());

