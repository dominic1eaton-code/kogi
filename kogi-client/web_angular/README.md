• 50+ reusable UI components

Navigator Prompt (interpreted):
Generate a **library of 50+ reusable UI components** for the **Kogi platform**, suitable for integration into the AngularJS app. Components should follow the same **dark theme design system** used in the uploaded login, onboarding, and dashboard screens and be written in **HTML + CSS class patterns compatible with AngularJS (`ng-*`)**.

Below is a **structured component library** you can drop into your AngularJS views.

---

# KOGI UI COMPONENT LIBRARY (v1)

Total components: **56**

Grouped into:

```
Layout
Navigation
Forms
Cards
Data
Interaction
Feedback
AI / Platform-specific
Governance / Marketplace
```

---

# 1. Layout Components

### 1. Page Container

```html
<div class="k-page">
  <div class="k-page-content">
    <!-- content -->
  </div>
</div>
```

---

### 2. Grid Layout

```html
<div class="k-grid k-grid-3">
  <div class="k-col"></div>
  <div class="k-col"></div>
  <div class="k-col"></div>
</div>
```

Variants

```
k-grid-2
k-grid-3
k-grid-4
```

---

### 3. Section Card

```html
<div class="k-section-card">
  <div class="k-section-header">
    <h3>Title</h3>
  </div>

  <div class="k-section-body">
    content
  </div>
</div>
```

---

### 4. Split Layout

```html
<div class="k-split">
  <div class="k-split-left"></div>
  <div class="k-split-right"></div>
</div>
```

---

# 2. Navigation Components

### 5. Sidebar Navigation

```html
<nav class="k-sidebar">

  <div class="k-nav-item active">
    <span class="k-nav-icon">◇</span>
    Dashboard
  </div>

</nav>
```

---

### 6. Navigation Badge

```html
<span class="k-badge red">3</span>
```

Variants

```
blue
green
purple
amber
```

---

### 7. Breadcrumb

```html
<div class="k-breadcrumb">
  <span>Home</span>
  <span>›</span>
  <span>Portfolio</span>
</div>
```

---

### 8. Tabs

```html
<div class="k-tabs">

  <div class="k-tab active">Projects</div>
  <div class="k-tab">Invoices</div>
  <div class="k-tab">Files</div>

</div>
```

---

### 9. Topbar Action Button

```html
<button class="k-btn k-btn-primary">
  + New Project
</button>
```

---

# 3. Form Components

### 10. Input Field

```html
<div class="k-field">

<label>Email</label>

<input
type="email"
ng-model="user.email">

</div>
```

---

### 11. Textarea

```html
<textarea class="k-textarea"></textarea>
```

---

### 12. Select Dropdown

```html
<select class="k-select">

<option>Freelancer</option>
<option>Agency</option>

</select>
```

---

### 13. Toggle Switch

```html
<label class="k-toggle">

<input type="checkbox">

<span class="k-toggle-slider"></span>

</label>
```

---

### 14. Checkbox

```html
<label class="k-checkbox">

<input type="checkbox">
<span></span>

Remember me

</label>
```

---

### 15. Radio Group

```html
<div class="k-radio-group">

<label>
<input type="radio" name="plan">
Starter
</label>

<label>
<input type="radio" name="plan">
Pro
</label>

</div>
```

---

### 16. File Upload

```html
<div class="k-upload">

<input type="file">

Drop files here

</div>
```

---

# 4. Buttons

### 17. Primary Button

```html
<button class="k-btn k-btn-primary">
Continue
</button>
```

---

### 18. Secondary Button

```html
<button class="k-btn k-btn-secondary">
Cancel
</button>
```

---

### 19. Icon Button

```html
<button class="k-btn-icon">
⚙
</button>
```

---

### 20. Floating Action Button

```html
<button class="k-fab">
+
</button>
```

---

# 5. Cards

### 21. Metric Card

```html
<div class="k-metric-card blue">

<div class="k-metric-number">
6
</div>

<div class="k-metric-label">
Active Projects
</div>

</div>
```

---

### 22. Project Card

```html
<div class="k-project-card">

<h4>Brand Redesign</h4>

<div class="k-progress">
  <div class="k-progress-bar" style="width:70%"></div>
</div>

</div>
```

---

### 23. Profile Card

```html
<div class="k-profile-card">

<div class="k-avatar">JD</div>

<h4>Jordan Davis</h4>

<p>Product Designer</p>

</div>
```

---

### 24. Marketplace Listing

```html
<div class="k-listing-card">

<h4>Website Design</h4>

<span class="k-price">$2,000</span>

</div>
```

---

# 6. Data Components

### 25. Table

```html
<table class="k-table">

<tr>
<th>Project</th>
<th>Status</th>
<th>Progress</th>
</tr>

<tr>
<td>Brand Identity</td>
<td>On Track</td>
<td>78%</td>
</tr>

</table>
```

---

### 26. Data Row

```html
<div class="k-data-row">

<span>Wallet Balance</span>

<strong>$4,820</strong>

</div>
```

---

### 27. Activity Feed

```html
<div class="k-feed">

<div class="k-feed-item">
Jordan completed a task
</div>

</div>
```

---

### 28. Timeline

```html
<div class="k-timeline">

<div class="k-timeline-item">
Project Started
</div>

</div>
```

---

# 7. Interaction Components

### 29. Modal

```html
<div class="k-modal">

<div class="k-modal-content">

<h3>New Project</h3>

</div>

</div>
```

---

### 30. Dropdown Menu

```html
<div class="k-dropdown">

<button>Menu</button>

<div class="k-dropdown-menu">

<div class="k-dropdown-item">
Edit
</div>

</div>

</div>
```

---

### 31. Tooltip

```html
<span class="k-tooltip">

Hover me

<span class="k-tooltip-text">
More info
</span>

</span>
```

---

### 32. Popover

```html
<div class="k-popover">
AI Suggestions
</div>
```

---

# 8. Feedback Components

### 33. Alert

```html
<div class="k-alert k-alert-warning">
Invoice overdue
</div>
```

Variants

```
success
danger
info
warning
```

---

### 34. Toast Notification

```html
<div class="k-toast">
Project saved
</div>
```

---

### 35. Progress Bar

```html
<div class="k-progress">

<div class="k-progress-bar"
style="width:60%">

</div>

</div>
```

---

### 36. Skeleton Loader

```html
<div class="k-skeleton"></div>
```

---

# 9. AI Components (Platform-Specific)

### 37. AI Suggestion Card

```html
<div class="k-ai-card">

⚠ Unblock 2 stories

</div>
```

---

### 38. AI Chat Window

```html
<div class="k-ai-chat">

<div class="k-ai-message user">
Hello
</div>

<div class="k-ai-message ai">
How can I help?
</div>

</div>
```

---

### 39. AI Prompt Input

```html
<div class="k-ai-input">

<input placeholder="Ask AI Agent...">

<button>Send</button>

</div>
```

---

# 10. Governance Components

### 40. Proposal Card

```html
<div class="k-proposal-card">

<h4>Fund Marketing Initiative</h4>

<span>Voting Open</span>

</div>
```

---

### 41. Voting Buttons

```html
<div class="k-vote">

<button class="k-vote-yes">
Yes
</button>

<button class="k-vote-no">
No
</button>

</div>
```

---

### 42. Vote Progress

```html
<div class="k-vote-bar">

<div class="k-vote-yes-bar"
style="width:62%">
</div>

</div>
```

---

# 11. Marketplace Components

### 43. Service Card

```html
<div class="k-service-card">

<h4>UI Design Package</h4>

<span>$1200</span>

</div>
```

---

### 44. Order Card

```html
<div class="k-order-card">

Order #2042

<span>Pending</span>

</div>
```

---

### 45. Wallet Card

```html
<div class="k-wallet-card">

<h4>Balance</h4>

<strong>$4,820</strong>

</div>
```

---

### 46. Transaction Row

```html
<div class="k-transaction">

<span>Client Payment</span>

<strong>$800</strong>

</div>
```

---

# 12. Portfolio Components

### 47. Portfolio Tile

```html
<div class="k-portfolio">

<img src="project.jpg">

<h4>Brand Identity</h4>

</div>
```

---

### 48. Skill Tag

```html
<span class="k-tag">
UI Design
</span>
```

---

### 49. Rating Component

```html
<div class="k-rating">
★★★★★
</div>
```

---

# 13. Utility Components

### 50. Divider

```html
<hr class="k-divider">
```

---

### 51. Avatar

```html
<div class="k-avatar">
JD
</div>
```

---

### 52. Status Dot

```html
<span class="k-dot green"></span>
```

---

### 53. Label Chip

```html
<span class="k-chip">
Pro Plan
</span>
```

---

### 54. Empty State

```html
<div class="k-empty">

No projects yet

<button class="k-btn k-btn-primary">
Create Project
</button>

</div>
```

---

### 55. Loading Spinner

```html
<div class="k-spinner"></div>
```

---

### 56. Keyboard Shortcut Badge

```html
<span class="k-key">⌘K</span>
```

---

# What This Gives You

You now have a **complete UI component system** suitable for the Kogi platform:

```
56 reusable UI components
consistent design language
Angular-compatible HTML
dark SaaS UI system
```

These components support the entire **Kogi platform vision**:

* Portfolio OS
* Marketplace
* Funding
* Governance
* AI agent
* Work management
* Analytics

---

If you'd like, I can also generate the **next layer that real platforms use**:

**1️⃣ 120+ advanced components**
(tables, kanban, gantt, wallet, DAO voting)

**2️⃣ The complete Kogi design system**
(tokens + spacing + typography)

**3️⃣ 70 real application screens**
(full SaaS product)

**4️⃣ AngularJS component directives**
so each UI element becomes:

```
<kogi-card>
<kogi-sidebar>
<kogi-metric>
```

instead of raw HTML.

That turns Kogi into a **true front-end framework instead of loose pages.**
