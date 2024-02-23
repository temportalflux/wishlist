---
created: 2024-02-23T20:48:43.584Z
updated: 2024-02-23T22:54:38.928Z
assigned: ""
progress: 0
tags:
  - Feature
started: 2024-02-22T00:00:00.000Z
---

# Spec out invite/invitee gifter functionality

A list's subtitle area contains a display showing the number of invited gifters and a button to add more.
Clicking on the invite button to add will open a popup to enter the username or email address of the user. If typing a non-email, the popup checks github to determine if its a valid user. If an emial is provided, the popup checks github for a user with that email. If no user is found, an email is sent to the recipient, inviting them to the platform. If a user is found, an email is sent to them to invite them to the inviter's list. Either way, the user/email is added to the list of invited users for the list, allowing them access to view it when they access the link.

When a user opens a list that they are invited to, we must check if they have acknowledged the invite. If they have not accepted the invite yet, then we prompt them to, which adds the list to their user info of "lists im invited to". If a user tries to access a list they are not in the "invited" list (allowlist), they are prevented from accessing the document. Lists a user has accepted invites to are downloaded when the account sync occurs (unless they have been removed from the allowlist).

A user can view who they've invited, and revoke access at any time. Revoking access prevents new updates from being downloaded by that user, and prevents them from sending reservations (modifying the document). Users revoked from accessing a list can be added back at any time.

Wishlists that the local user has been invited to view show up on their homepage separately from lists they own. Lists on the homepage show the name of the user who owns the list (ideally the display name, not username).
