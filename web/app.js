// Populates the download section from the repository's real releases.
//
// The page never hardcodes a version or a file name: it asks GitHub what the
// latest release is and renders what is actually attached to it. That way the
// page cannot advertise a build that was never published, and it cannot go
// stale when one is.
//
// No dependencies, no build step. `fetch` and the DOM.

(function () {
  "use strict";

  var REPO = "Lucas-Belucci-Bellini/NEXORA";
  var LATEST = "https://api.github.com/repos/" + REPO + "/releases/latest";
  var RELEASES_PAGE = "https://github.com/" + REPO + "/releases";

  // Maps the label the release workflow puts in each archive name to something
  // a person recognises, plus how to guess whether it is the one they want.
  var PLATFORMS = [
    {
      label: "linux-x86_64",
      name: "Linux",
      detail: "x86-64",
      matches: function (ua) { return /linux/i.test(ua) && !/android/i.test(ua); }
    },
    {
      label: "windows-x86_64",
      name: "Windows",
      detail: "x86-64",
      matches: function (ua) { return /win/i.test(ua); }
    },
    {
      label: "macos-aarch64",
      name: "macOS",
      detail: "Apple silicon",
      matches: function (ua) { return /mac/i.test(ua); }
    }
  ];

  var status = document.getElementById("release-status");
  var grid = document.getElementById("release-grid");
  var verify = document.getElementById("verify");

  function say(message, isProblem) {
    status.textContent = "";
    status.append(message);
    status.hidden = false;
    status.classList.toggle("problem", Boolean(isProblem));
  }

  // Build a DOM fragment rather than assigning innerHTML: release names and
  // asset names come from an API, and this page has no business interpreting
  // any of it as markup.
  function element(tag, className, text) {
    var node = document.createElement(tag);
    if (className) { node.className = className; }
    if (text !== undefined) { node.textContent = text; }
    return node;
  }

  function readableSize(bytes) {
    if (typeof bytes !== "number" || !isFinite(bytes) || bytes < 0) { return null; }
    var mib = bytes / (1024 * 1024);
    return (mib >= 10 ? Math.round(mib) : Math.round(mib * 10) / 10) + " MiB";
  }

  function assetFor(assets, label) {
    for (var i = 0; i < assets.length; i += 1) {
      var name = assets[i].name || "";
      // The workflow names archives `nexora-<version>-<label>.<ext>`; the
      // checksum sidecars are not offered as downloads here.
      if (name.indexOf(label) !== -1 && !/\.sha256$/.test(name)) {
        return assets[i];
      }
    }
    return null;
  }

  function likelyPlatform() {
    var ua = navigator.userAgent || "";
    for (var i = 0; i < PLATFORMS.length; i += 1) {
      if (PLATFORMS[i].matches(ua)) { return PLATFORMS[i].label; }
    }
    return null;
  }

  function card(platform, asset, isLikely) {
    var node = element("div", "asset" + (isLikely ? " here" : ""));
    if (isLikely) {
      node.append(element("span", "for-you", "Probably yours"));
    }
    node.append(element("span", "platform", platform.name));
    node.append(element("span", "meta", platform.detail));

    if (!asset) {
      node.append(element("span", "meta", "Not in this release."));
      return node;
    }

    var size = readableSize(asset.size);
    node.append(element("span", "filename", asset.name));
    if (size) { node.append(element("span", "meta", size)); }

    var link = element("a", "button primary", "Download");
    link.href = asset.browser_download_url;
    link.setAttribute("rel", "noopener");
    node.append(link);
    return node;
  }

  function render(release) {
    var assets = Array.isArray(release.assets) ? release.assets : [];
    var mine = likelyPlatform();
    var offered = 0;

    grid.textContent = "";
    PLATFORMS.forEach(function (platform) {
      var asset = assetFor(assets, platform.label);
      if (asset) { offered += 1; }
      grid.append(card(platform, asset, platform.label === mine));
    });

    if (offered === 0) {
      // A release exists but carries no archive this page knows how to offer.
      // Saying so is better than rendering three empty cards.
      grid.hidden = true;
      var none = element("span", null, "The latest release has no archives attached. ");
      var link = element("a", null, "Open it on GitHub");
      link.href = release.html_url || RELEASES_PAGE;
      say(none, true);
      status.append(link);
      status.append(" to see what it contains.");
      return;
    }

    grid.hidden = false;
    verify.hidden = false;

    var name = release.tag_name || release.name || "the latest release";
    var line = element("span", null, "Latest release: ");
    var tag = element("strong", null, name);
    say(line, false);
    status.append(tag);
    if (release.published_at) {
      var when = new Date(release.published_at);
      if (!isNaN(when.getTime())) {
        status.append(", published " + when.toISOString().slice(0, 10) + ". ");
      } else {
        status.append(". ");
      }
    } else {
      status.append(". ");
    }
    var all = element("a", null, "All releases");
    all.href = RELEASES_PAGE;
    status.append(all);
    status.append(".");
  }

  function noReleaseYet() {
    grid.hidden = true;
    verify.hidden = true;
    var lead = element("span", null,
      "No build has been published yet. The release pipeline is in place and " +
      "publishes on a version tag — until then, ");
    say(lead, false);
    var link = element("a", null, "build it from source");
    link.href = "#source";
    status.append(link);
    status.append(", which is three commands and no dependencies.");
  }

  function unreachable() {
    grid.hidden = true;
    verify.hidden = true;
    var lead = element("span", null,
      "Could not reach the GitHub API from here — it may be rate-limiting this " +
      "address. ");
    say(lead, true);
    var link = element("a", null, "Open the releases page directly");
    link.href = RELEASES_PAGE;
    status.append(link);
    status.append(".");
  }

  if (!window.fetch) {
    grid.hidden = true;
    var lead = element("span", null, "This browser cannot load the release list. ");
    say(lead, true);
    var link = element("a", null, "Open the releases page");
    link.href = RELEASES_PAGE;
    status.append(link);
    return;
  }

  fetch(LATEST, { headers: { Accept: "application/vnd.github+json" } })
    .then(function (response) {
      // 404 is the honest answer for a repository with no releases, and it is
      // a state this page expects rather than an error to report.
      if (response.status === 404) { noReleaseYet(); return null; }
      if (!response.ok) { throw new Error("HTTP " + response.status); }
      return response.json();
    })
    .then(function (release) {
      if (release) { render(release); }
    })
    .catch(function () {
      unreachable();
    });
})();
