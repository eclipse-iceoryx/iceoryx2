// Copyright (c) 2026 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

// Turns CSS transitions off while the theme is switched, otherwise colors fade between
// the themes and flicker.
{
  const root = document.documentElement;

  const afterNextPaint = (callback) => {
    requestAnimationFrame(() => requestAnimationFrame(callback));
  };

  const suppress = () => {
    root.classList.add("ix-theme-switching");
    afterNextPaint(() => root.classList.remove("ix-theme-switching"));
  };

  document.addEventListener(
    "click",
    (event) => {
      if (event.target.closest(".theme-toggle")) suppress();
    },
    true,
  );

  window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", suppress);
}
