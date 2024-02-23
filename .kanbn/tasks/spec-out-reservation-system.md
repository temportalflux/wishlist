---
created: 2024-02-23T22:35:11.610Z
updated: 2024-02-23T22:53:43.381Z
assigned: ""
progress: 0
tags:
  - Feature
---

# Spec out reservation system

Users invited to access another user's list have different capabilities than the owner. They cannot rearrange items, edit their details, delete them, or do anything that would change the list. Unlike the owner, they are shown if an item is reserved or not (if it is reserved, then is it 1/1 or 1/x), and if there are available reservations, they can opt to mark the item as reserved by them.
When the owner of a list creates an item, the reservation capacity defaults to 1. They can specify value to be any quantity > 1, or say that there is no max capacity.
Reserving an item does edit the wishlist document, adding the user to the "reservations" list and the amount they reserved.
The owner of a list can customize a list to specify if they can a) see that items are reserved and b) who reserved them. By default, both of these are disabled for new lists.
TBD: how are reservations resolved, esp when the owner chooses to not view reservations, without requiring reserver's to go back to the app after delivery to resolve the entry?
