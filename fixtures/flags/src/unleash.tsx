export const Toolbar = () => (useFlag("un-js") ? <NewToolbar /> : <OldToolbar />);
