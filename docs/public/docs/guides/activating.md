# Activating Your Appliance

By the end of this page, you'll know how to activate a KruxOS appliance online with a pairing code, how offline activation works, and how seats behave when you move or re-image an appliance.

!!! note "Activation never locks you out"
    Activation is a soft check. An appliance with a missing, expired, or invalid
    license keeps working — you'll see a banner in the dashboard, not a lockout.

## Online activation (recommended)

When the appliance asks you to activate — in the first-boot wizard, in **Dashboard → Settings → License**, or via the admin CLI — it shows an **8-character pairing code**.

1. Go to [kruxos.com/activate](https://kruxos.com/activate) on any browser (your phone works fine).
2. Sign in with your email — see [Signing In](signing-in.md).
3. Type the 8-character code shown on your appliance and choose the license to use.
4. The appliance picks up the approval automatically within a few seconds and stores its license key.

The appliance never needs your account credentials — only the pairing code travels between the two.

## Offline activation

If the appliance has no internet access, you can activate by pasting the license key directly:

1. Sign in at [kruxos.com](https://kruxos.com) and download your license key from your account page.
2. In **Dashboard → Settings → License**, choose the offline option and paste the key (the first-boot wizard offers the same path).

Enterprise licenses are activated online only — [contact sales](mailto:sales@altvale.com) if your Enterprise deployment is air-gapped.

## Seats — one appliance per seat

Each license activation binds one **seat** to one appliance. The seat survives reboots and updates — you never re-activate for normal operation.

**Moving to a new appliance:** a seat is freed only when you explicitly deactivate the old appliance from your account page at [kruxos.com](https://kruxos.com). If you activate a new appliance while the seat is still bound, you'll see **"This license is already in use"** — deactivate the old appliance in your account, then enter the code again (or get an additional key, where your tier allows it).

**Re-imaging:** a full re-install counts as a new appliance. Deactivate the old seat in your account (before or after re-imaging), then activate the fresh install as normal.

## Still stuck?

[Contact support](https://kruxos.com/contact/) with the appliance's pairing code and the email on your account.

## Related

- [Signing In to Your KruxOS Account](signing-in.md) — magic-link sign-in help
- [First-Boot Wizard](first-boot-wizard.md) — activation during initial setup
- [Pricing](../enterprise/pricing.md) — tiers and licensing
