import { AnimatePresence, motion } from "framer-motion";
import { useState, useEffect } from "react";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

import { invoke } from "@tauri-apps/api/tauri";

export const SetupPage = () => {
  const [show, setShow] = useState(true);

  useEffect(() => {
    // Set a timeout to hide the message after 3 seconds
    const timer = setTimeout(() => {
      setShow(false);
    }, 1350); // Adjust the time as needed

    // Clear the timeout when the component unmounts
    return () => clearTimeout(timer);
  }, []);

  return (
    <main className="h-dvh flex flex-col justify-center items-center">
      <AnimatePresence>
        {show && (
          <motion.span
            className="text-6xl"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.5 }} // Controls the fade-in and fade-out duration
          >
            Welcome
          </motion.span>
        )}
      </AnimatePresence>
      <Card className="w-[65%] max-w-[500px]">
        <CardHeader>
          <CardTitle className="text-2xl">Choose Your Root Folder</CardTitle>
          <CardDescription>
            The search scope will be limited to this folder
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form>
            <div className="flex flex-col space-y-1.5">
              <Label htmlFor="name">Root Folder:</Label>
              <Input id="name" placeholder="Enter the root folder here" />
            </div>
          </form>
        </CardContent>
      </Card>
    </main>
  );
};
