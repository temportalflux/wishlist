---
created: 2024-02-24T20:22:28.737Z
updated: 2024-02-24T22:48:23.830Z
assigned: ""
progress: 0
tags:
  - feature
---

# Implement touch-based drag & drop

yew-hooks doesnt simulate drag & drop events for touch devices. Will need to write a custom wrapper to handle this.
https://www.npmjs.com/package/drag-drop-touch?activeTab=code is an OK example for dispatching drag and drop start/end events based on touch events.
https://docs.rs/yew-hooks/latest/src/yew_hooks/hooks/use_drag.rs.html#89 is a good example for implementing event listener hooks in rust/yew.

## Relations

- [supports reorder-items-in-list-using-drag-drop](reorder-items-in-list-using-drag-drop.md)
