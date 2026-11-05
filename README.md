# jrussdev — localswebbuilds.xyz

Single-file landing page for the jrussdev freelance web development brand.

---

## Files

| File | Purpose |
|------|---------|
| `index.html` | The entire site — HTML, CSS, and JS in one file |

---

## Quick start

1. Download `index.html`
2. Upload it to your hosting (Cloudflare Pages, Netlify, or any static host)
3. Point your domain `localswebbuilds.xyz` to the host
4. Complete the two steps below before going live

---

## Setup checklist

### 1. Formspree (contact form)

1. Go to [formspree.io](https://formspree.io) and create a free account
2. Create a new form and copy your endpoint URL (looks like `https://formspree.io/f/abcd1234`)
3. Open `index.html` and find the `CONFIG` block near the bottom of the `<script>` tag:

```js
const CONFIG = {
  FORMSPREE_ENDPOINT: 'https://formspree.io/f/YOUR_FORM_ID', // replace this
  REXDEV_EMAIL:       'hello@localswebbuilds.xyz',
};
```

Replace `YOUR_FORM_ID` with your actual ID.

---

### 2. Portfolio entries

Find the `PORTFOLIO` array in the `<script>` block:

```js
const PORTFOLIO = [
  {
    name:        'Example Business 1',
    type:        'Local business',
    url:         'https://example.com',      // replace with real URL
    description: 'Placeholder description.',  // replace with real description
  },
  // ...
];
```

- Replace `url` with the live URL of your client's site
- Screenshots load **automatically** via [thum.io](https://thum.io) — no image uploads needed
- Add or remove objects in the array to change the number of cards

---

## Email routing

Already configured via Cloudflare Email Routing:
- `hello@localswebbuilds.xyz` forwards to `easybuildweb3@gmail.com`

No changes needed in the HTML — the email address is already set correctly.

---

## Deployment options

| Host | Free tier | Custom domain | Notes |
|------|-----------|--------------|-------|
| [Cloudflare Pages](https://pages.cloudflare.com) | Yes | Yes | Best option — integrates with your existing Cloudflare setup |
| [Netlify](https://netlify.com) | Yes | Yes | Drag-and-drop deploy |
| [GitHub Pages](https://pages.github.com) | Yes | Yes | Push to repo and enable Pages |

### Cloudflare Pages (recommended)

1. Cloudflare Dashboard → Pages → Create a project
2. Choose "Direct Upload"
3. Upload `index.html`
4. Connect your custom domain `localswebbuilds.xyz`

---

## Customisation reference

All key config is near the bottom of the `<script>` block:

```js
const CONFIG = { ... };   // Formspree endpoint + contact email
const PORTFOLIO = [ ... ]; // Portfolio cards
```

Colours are CSS variables at the top of the `<style>` block:

```css
:root {
  --primary:  #1e293b;   /* deep slate */
  --accent:   #3b82f6;   /* blue — buttons, highlights */
}
```

---

## Fonts

Uses **Plus Jakarta Sans** via Google Fonts — loads automatically online. A fallback sans-serif shows when previewing offline; that's expected.

---

*Built by jrussdev · hello@localswebbuilds.xyz*
