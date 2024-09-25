import {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableFooter,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

export const RecentsContainer = ({ className, recent }) => {
  className += " col-span-7 p-2";
  const items = recent.getItems();

  return (
    <section className={className}>
      <div>
        <span className="text-2xl">Recent Files</span>
      </div>
      {items.length === 0 && (
        <p className="text-muted-foreground">No recently opened files</p>
      )}
      {items.length > 0 && (
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="">File Name</TableHead>
              <TableHead>File Type</TableHead>
              <TableHead>File Size</TableHead>
              <TableHead className="">Creation Date</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {items.map((file, index) => (
              <TableRow key={index} onClick={() => openFile(file)}>
                <TableCell className="font-medium">
                  {file["fileName"]}
                </TableCell>
                <TableCell>{file["fileType"]}</TableCell>
                <TableCell>{file["fileSize"]} B</TableCell>
                <TableCell className="">{file["formattedDate"]}</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      )}
    </section>
  );
};
