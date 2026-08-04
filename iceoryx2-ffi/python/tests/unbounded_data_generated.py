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

# pylint: skip-file
# mypy: ignore-errors

import flatbuffers
from flatbuffers.compat import import_numpy

np = import_numpy()


class UnboundedData(object):
    __slots__ = ["_tab"]

    @classmethod
    def GetRootAs(cls, buf, offset=0):
        n = flatbuffers.encode.Get(flatbuffers.packer.uoffset, buf, offset)
        x = UnboundedData()
        x.Init(buf, n + offset)
        return x

    @classmethod
    def GetRootAsUnboundedData(cls, buf, offset=0):
        """This method is deprecated. Please switch to GetRootAs."""
        return cls.GetRootAs(buf, offset)

    # UnboundedData
    def Init(self, buf, pos):
        self._tab = flatbuffers.table.Table(buf, pos)

    # UnboundedData
    def Data1(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Int32Flags, o + self._tab.Pos)
        return 0


def UnboundedDataStart(builder):
    builder.StartObject(1)


def Start(builder):
    UnboundedDataStart(builder)


def UnboundedDataAddData1(builder, data1):
    builder.PrependInt32Slot(0, data1, 0)


def AddData1(builder, data1):
    UnboundedDataAddData1(builder, data1)


def UnboundedDataEnd(builder):
    return builder.EndObject()


def End(builder):
    return UnboundedDataEnd(builder)


def create_unbounded_data(builder, data_1):
    UnboundedDataStart(builder)
    UnboundedDataAddData1(builder, data_1)
    return UnboundedDataEnd(builder)
