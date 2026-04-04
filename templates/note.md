---
title: "{{ title }}"
description: "{{ description | assigned_or("") }}"
{% if tags.is_empty() %}
tags: []
{% else %}
tags:
  {% for tag in tags %}
    - "{{ tag }}"
  {% endfor %}
{% endif %}
created: "{{ created_at }}"
{% if let Some(updated_at) = updated_at %}
updated: "{{ updated_at }}"
{% endif %}
language: "{{ lang | assigned_or("eng") }}"
---

# {{ title }}

{#
vim:ft=jinja.markdown
#}
