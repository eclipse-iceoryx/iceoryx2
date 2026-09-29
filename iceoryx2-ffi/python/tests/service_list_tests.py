# Copyright (c) 2026 Contributors to the Eclipse Foundation
#
# See the NOTICE file(s) distributed with this work for additional
# information regarding copyright ownership.
#
# This program and the accompanying materials are made available under the
# terms of the Apache Software License 2.0 which is available at
# https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
# which is available at https://opensource.org/licenses/MIT.
#
# SPDX-License-Identifier: Apache-2.0 OR MIT

import stat
from pathlib import Path

import iceoryx2 as iox2
import pytest


def test_list_reports_corrupt_service_details() -> None:
    config = iox2.testing.generate_isolated_config()
    node = iox2.NodeBuilder.new().config(config).create(iox2.ServiceType.Ipc)
    service_name = iox2.testing.generate_service_name()
    service = node.service_builder(service_name).event().create()

    services = iox2.Service.list(config, iox2.ServiceType.Ipc)
    assert len(services) == 1
    assert services[0].name() == service_name

    service_dir = Path(config.global_cfg.service_dir.to_string())
    prefix = config.global_cfg.prefix.to_string()
    suffix = config.global_cfg.service.static_config_storage_suffix.to_string()
    static_files = list(service_dir.glob(f"{prefix}*{suffix}"))
    assert len(static_files) == 1
    static_file = static_files[0]
    original = static_file.read_bytes()
    original_mode = stat.S_IMODE(static_file.stat().st_mode)
    try:
        static_file.chmod(original_mode | stat.S_IWUSR)
        static_file.write_bytes(b"\0" * len(original))
        static_file.chmod(original_mode)
        with pytest.raises(iox2.ServiceDetailsError) as details_error:
            iox2.Service.details(
                service_name, config, iox2.MessagingPattern.Event, iox2.ServiceType.Ipc
            )
        assert "FailedToDeserializeStaticServiceInfo" in str(details_error.value)

        with pytest.raises(iox2.ServiceListError) as list_error:
            iox2.Service.list(config, iox2.ServiceType.Ipc)
        assert "FailedToDeserializeStaticServiceInfo" in str(list_error.value)
    finally:
        static_file.chmod(original_mode | stat.S_IWUSR)
        static_file.write_bytes(original)
        static_file.chmod(original_mode)

    assert len(iox2.Service.list(config, iox2.ServiceType.Ipc)) == 1
    del service
    del node
