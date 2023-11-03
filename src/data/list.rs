pub struct List {
	pub name: String,
	// user-ids of those whove been invited to access this list (in addition to the owner)
	pub invitees: Vec<String>,
}
