/*
 * Copyright 2018 astonbitecode
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
package org.astonbitecode.j4rs.api.invocation;

import org.astonbitecode.j4rs.api.NativeInvocation;
import org.astonbitecode.j4rs.api.dtos.InvocationArg;
import org.astonbitecode.j4rs.errors.InvocationException;
import org.astonbitecode.j4rs.utils.ChildDummy;
import org.astonbitecode.j4rs.utils.Dummy;
import org.astonbitecode.j4rs.utils.DummyWithStatic;
import org.astonbitecode.j4rs.utils.FailingDummy;
import org.junit.Test;

public class JsonInvocationImplTest {
    @Test
    public void methodMatches() {
        JsonInvocationImpl toTest = new JsonInvocationImpl(new Dummy(33), Dummy.class);

        NativeInvocation invocation = toTest.invoke("getI");
        assert (invocation.getObject().getClass().equals(Integer.class));
        assert (((Integer) invocation.getObject()) == 33);
    }

    @Test
    public void staticMethodMatches() {
        JsonInvocationImpl toTest = new JsonInvocationImpl(DummyWithStatic.class);

        NativeInvocation invocation = toTest.invokeStatic("method");
        assert (invocation.getObject().getClass().equals(String.class));
        assert (((String) invocation.getObject()).equals("method product"));
    }

    @Test
    public void withArg() {
        JsonInvocationImpl toTest = new JsonInvocationImpl(new Dummy(33), Dummy.class);

        InvocationArg arg = new InvocationArg(new JsonInvocationImpl(3, Integer.class));
        toTest.invoke("setI", arg);

        NativeInvocation getIResult = toTest.invoke("getI");
        assert (getIResult.getObject().getClass().equals(Integer.class));
        assert (((Integer) getIResult.getObject()) == 3);
    }

    @Test
    public void staticMethodWithArg() {
        JsonInvocationImpl toTest = new JsonInvocationImpl(DummyWithStatic.class);

        InvocationArg arg = new InvocationArg(new JsonInvocationImpl(3, Integer.class));
        NativeInvocation invocation = toTest.invokeStatic("methodWithArg", arg);

        assert (invocation.getObject().getClass().equals(String.class));
        assert (((String) invocation.getObject()).equals("3"));
    }

    @Test(expected = Exception.class)
    public void noMethodFound() {
        new JsonInvocationImpl(new Dummy(33), Dummy.class).invoke("nonExisting");
    }

    @Test(expected = Exception.class)
    public void errorDuringInvocation() {
        new JsonInvocationImpl(new FailingDummy(), FailingDummy.class).invoke("throwException");
    }

    @Test
    public void cast() {
        NativeInvocation from = new JsonInvocationImpl(new ChildDummy(), ChildDummy.class);
        NativeInvocation casted = NativeInvocation.<Dummy>cast(from, Dummy.class.getName());
//        assert(casted.getObject().getClass().equals(Dummy.class));
    }

    @Test (expected = InvocationException.class)
    public void castFailure() {
        NativeInvocation from = new JsonInvocationImpl(new ChildDummy(), ChildDummy.class);
        NativeInvocation.cast(from, Integer.class.getName());
    }
}
