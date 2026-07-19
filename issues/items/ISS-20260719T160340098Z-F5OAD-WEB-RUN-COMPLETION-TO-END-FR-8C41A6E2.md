---
id: ISS-20260719T160340098Z-F5OAD-WEB-RUN-COMPLETION-TO-END-FR-8C41A6E2
title: "F5oad Web Run completion to EndFrame command"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-20
updated: 2026-07-20
target: stdlib/platforms/gui/web/font_registered_end_frame_command.nepl
---

# ISS-20260719T160340098Z-F5OAD-WEB-RUN-COMPLETION-TO-END-FR-8C41A6E2: F5oad Web Run completion to EndFrame command

## 概要

F5oac actual Run success retains the exact Run cursor and Continue completion but cannot reuse F5nzq without reconstructing consumed schedule authority.

## 修正方針

Consume F5oac success once, preserve retry provenance and spent budget, and pass only the post-Run Continue state and exact cursor into a module-private transition implementing the same F5nzq finish/step contract. Do not replay Run scheduling/request/execution or execute EndFrame.

## 検証

Actual Web fixtures return evidence 94 with Begin 2 / Run 2 / End 0, including Continue success and Yield recovery 31. Source policy verifies module-private raw transitions, a single cursor finish/step transition, no schedule-owner constructor, no schedule/request replay, and no platform execution in the adapter.
