import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ItembookLogsComponent } from './itembook-logs.component';

describe('ItembookLogsComponent', () => {
  let component: ItembookLogsComponent;
  let fixture: ComponentFixture<ItembookLogsComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ItembookLogsComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(ItembookLogsComponent);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
